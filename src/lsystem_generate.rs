use rand::prelude::*;
use std::collections::HashMap;

use super::*;

impl LSystem {
    pub fn new(start: String, rule_map: HashMap<char, String>, angle: f64) -> Self {
        LSystem {
            seed: start.clone(),
            string: start,
            rules: rule_map,
            angle,
        }
    }

    fn grow(&mut self, max_len: usize) {
        let mut growth_cycle = 0;
        let mut grown_string = self.string.clone(); // FIXME: pretty inefficient to clone this here

        while grown_string.len() < max_len {
            self.string = grown_string.clone();
            let mut grown_string_arr = String::new();
            for ch in self.string.chars() {
                if let Some(string) = self.rules.get(&ch) {
                    grown_string_arr.push_str(string.as_str());
                } else {
                    grown_string_arr.push(ch)
                }
            }

            grown_string = grown_string_arr;
            if grown_string == self.string {
                break;
            }

            if growth_cycle > MAX_GROWTH_CYCLES {
                break;
            }

            growth_cycle += 1;
        }
    }
}

impl TurtleStates {
    pub fn new(params: &Parameters, rng: &mut ThreadRng) -> Self {
        let mut lsys = rand_lsystem(params, rng);
        lsys.grow(params.lsystem_max_length);
        TurtleStates {
            current_string_pos: 0,
            lsys,
        }
    }
}

impl Iterator for TurtleStates {
    type Item = Vec<CurrentString>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut num_fs = 0;
        let mut current_strings = Vec::new();

        while self.current_string_pos < self.lsys.string.len() {
            let next_f_pos = match self.lsys.string[self.current_string_pos..].find('F') {
                Some(pos) => {
                    num_fs += 1;
                    self.current_string_pos + pos
                }
                None => self.lsys.string.len() - 1,
            };

            let string: String = self.lsys.string[self.current_string_pos..next_f_pos + 1].into();
            let current_string = CurrentString {
                string,
                angle: self.lsys.angle,
            };

            current_strings.push(current_string);
            if num_fs >= FS_PER_TURTLE_MOVE {
                return Some(current_strings);
            }

            self.current_string_pos = next_f_pos + 1;
        }

        None
    }
}

// returns radians (not degrees)
fn rand_angle(params: &Parameters, rng: &mut ThreadRng) -> f64 {
    if chance(params.random_angle_chance, rng) {
        rng.gen_range(MIN_ANGLE, MAX_ANGLE)
    } else {
        *rand_choice(&NON_RANDOM_ANGLES, rng)
    }
}

struct CharSet {
    pub banned_chars: Vec<char>, // FIXME: turn this into a hashset
    pub set: Vec<char>,
}

impl CharSet {
    fn new(banned_chars: Vec<char>) -> Self {
        CharSet {
            banned_chars,
            set: Vec::new(),
        }
    }

    fn add_chars(&mut self, s: &String) {
        for c in s.chars() {
            if !self.banned_chars.contains(&c) {
                self.set.push(c)
            }
        }
    }

    fn ban_char(&mut self, c: char) {
        self.set.retain(|&x| c != x); // remove c from list
        self.banned_chars.push(c);
    }

    fn rand_char(&mut self, rng: &mut ThreadRng) -> Option<char> {
        if self.set.len() == 0 {
            None
        } else {
            Some(*rand_choice(&self.set, rng))
        }
    }
}

fn try_to_create_rule_map(
    start: &String,
    mut rule_strings: Vec<String>,
    rng: &mut ThreadRng,
) -> Option<HashMap<char, String>> {
    let mut used_chars = CharSet::new(vec!['[', ']']);
    used_chars.add_chars(&start);
    let mut all_rules = HashMap::new();

    loop {
        if let Some(rule_string) = rule_strings.pop() {
            if let Some(rule_key) = used_chars.rand_char(rng) {
                used_chars.ban_char(rule_key);
                used_chars.add_chars(&rule_string);
                all_rules.insert(rule_key, rule_string);
            } else {
                return None;
            }
        } else {
            break;
        }
    }

    Some(all_rules)
}

fn rand_lsystem(params: &Parameters, rng: &mut ThreadRng) -> LSystem {
    let angle = rand_angle(params, rng);
    let num_rules: usize = rng.gen_range(params.min_rules, params.max_rules);

    loop {
        let rule_strings = create_random_rule_strings(num_rules, params, rng);
        let num_start_chars: usize =
            rng.gen_range(params.min_start_length, params.max_start_length);
        let start = rand_lsystem_string(num_start_chars, rng);
        if let Some(rule_map) = try_to_create_rule_map(&start, rule_strings, rng) {
            return LSystem::new(start, rule_map, angle);
        }
    }
}

fn chance(percentage: f64, rng: &mut ThreadRng) -> bool {
    let num: f64 = rng.gen(); // 0 to 1
    num < percentage
}

fn rand_choice<'a, T>(array: &'a [T], rng: &mut ThreadRng) -> &'a T {
    array.iter().choose(rng).unwrap()
}

fn rand_choice_mut<'a, T>(array: &'a mut [T], rng: &mut ThreadRng) -> &'a mut T {
    array.iter_mut().choose(rng).unwrap()
}

fn rand_lsystem_string(len: usize, rng: &mut ThreadRng) -> String {
    const POSSIBLE_CHARS: [char; 5] = ['F', '+', '-', 'A', 'B'];
    const SQUARE_BRACKET_CHANCE: f64 = 1.0 / (POSSIBLE_CHARS.len() as f64 + 1.0);

    loop {
        let mut num_bracket_pairs = 0;
        for _ in 0..(len as f64 / 2.0).floor() as usize {
            if chance(SQUARE_BRACKET_CHANCE, rng) {
                num_bracket_pairs += 1;
            }
        }

        let mut string = String::new();
        let num_random_letters = len - (num_bracket_pairs * 2);
        for _ in 0..num_random_letters {
            let rand_char = rand_choice(&POSSIBLE_CHARS, rng);
            string.push(*rand_char);
        }

        if string.len() == 0 {
            continue;
        }

        let mut rng = rand::thread_rng();
        for _ in 0..num_bracket_pairs {
            let opening_location: usize = rng.gen_range(0, string.len());
            let closing_location: usize = rng.gen_range(opening_location, string.len());
            string.insert(opening_location, '['); // FIXME: check that this is correct, also, this is pretty inefficient
            string.insert(closing_location + 1, ']'); // FIXME: check that this is correct
        }

        return string;
    }
}

fn create_random_rule_strings(
    num_rules: usize,
    params: &Parameters,
    rng: &mut ThreadRng,
) -> Vec<String> {
    let mut rule_strings = Vec::new();
    for _ in 0..num_rules {
        let len = rng.gen_range(params.min_rule_length, params.max_rule_length);
        let rule_string = rand_lsystem_string(len, rng);
        rule_strings.push(rule_string);
    }

    let has_f = rule_strings.iter().any(|f| f.contains("F"));
    if !has_f {
        let rule_string = rand_choice_mut(&mut rule_strings, rng);
        let location: usize = rng.gen_range(0, rule_string.len());
        rule_string.insert(location, 'F');
    }

    rule_strings
}
