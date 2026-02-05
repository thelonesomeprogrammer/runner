use crate::model::Entry;
use crate::config::Config;
use crate::matcher::FuzzyMatcher;
use crate::sources::history::{self, History};
use regex::Regex;

pub struct AppState {

    pub config: Config,

    pub entries: Vec<Entry>,

    pub filtered_indices: Vec<usize>,

    pub selected_index: usize,

    pub query: String,

    pub matcher: FuzzyMatcher,

    pub active_group: String,

    pub history: History,

}



impl AppState {

    pub fn new(config: Config) -> Self {

        Self {

            config,

            entries: Vec::new(),

            filtered_indices: Vec::new(),

            selected_index: 0,

            query: String::new(),

            matcher: FuzzyMatcher::new(),

            active_group: "default".to_string(),

            history: history::load_history(),

        }

    }



    pub fn set_entries(&mut self, entries: Vec<Entry>) {

        self.entries = entries;

        self.update_filter();

    }



    pub fn update_query(&mut self, query: &str) {

        self.query = query.to_string();

        self.update_filter();

    }

    

    pub fn update_filter(&mut self) {

        let group_config = self.config.groups.get(&self.active_group);

        

        if self.query.is_empty() {

            // Sort original entries by history for the "empty query" state

            // We still need a list of all indices

            let mut indices: Vec<usize> = (0..self.entries.len()).collect();

            

            indices.sort_by(|&a, &b| {

                let a_entry = &self.entries[a];

                let b_entry = &self.entries[b];

                let a_count = self.history.usage_counts.get(&a_entry.id).unwrap_or(&0);

                let b_count = self.history.usage_counts.get(&b_entry.id).unwrap_or(&0);

                b_count.cmp(a_count).then_with(|| a_entry.name.cmp(&b_entry.name))

            });

            self.filtered_indices = indices;

        } else {

            // Update scores in place in the main entries list

            self.matcher.match_entries(&self.query, &mut self.entries);

            

            // Apply history boost

            for entry in self.entries.iter_mut() {

                if entry.score > 0 {

                    let count = self.history.usage_counts.get(&entry.id).unwrap_or(&0);

                    entry.score += (*count as i64) * 100;

                }

            }



            // Collect indices of matching entries

            let mut indices: Vec<usize> = self.entries.iter().enumerate()

                .filter(|(_, e)| e.score > 0)

                .map(|(i, _)| i)

                .collect();



            // Sort indices by entry score

            indices.sort_by(|&a, &b| {

                self.entries[b].score.cmp(&self.entries[a].score)

            });

            

            self.filtered_indices = indices;

        };



        // Apply Blacklist/Whitelist from Group

        if let Some(gc) = group_config {

            let mut to_remove = Vec::new();

            

            // Prepare regexes once

            let regexes: Vec<Regex> = gc.blacklist.as_ref()

                .map(|bl| bl.iter().filter_map(|s| Regex::new(s).ok()).collect())

                .unwrap_or_default();



            for (i, &idx) in self.filtered_indices.iter().enumerate() {

                let e = &self.entries[idx];

                

                // Whitelist check

                if let Some(whitelist) = &gc.whitelist {

                    if !whitelist.iter().any(|w| e.name.contains(w) || e.id.contains(w)) {

                        to_remove.push(i);

                        continue;

                    }

                }



                // Blacklist check

                if !regexes.is_empty() && regexes.iter().any(|re| re.is_match(&e.name) || re.is_match(&e.id)) {

                    to_remove.push(i);

                }

            }

            

            // Remove in reverse to maintain index validity

            for i in to_remove.into_iter().rev() {

                self.filtered_indices.remove(i);

            }

        }



        log::info!("AppState: query='{}', filtered_count={}", self.query, self.filtered_indices.len());

        self.selected_index = 0;

    }

    

    pub fn move_selection(&mut self, delta: i32) {

        if self.filtered_indices.is_empty() {

            self.selected_index = 0;

            return;

        }

        

        let len = self.filtered_indices.len() as i32;

        let new_index = (self.selected_index as i32 + delta).rem_euclid(len);

        self.selected_index = new_index as usize;

    }

    

    pub fn get_selected(&self) -> Option<&Entry> {

        self.filtered_indices.get(self.selected_index)

            .map(|&idx| &self.entries[idx])

    }

}
