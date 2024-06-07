use rand::prelude::*;



const LETTERS: &str = "abcdefghijklmnopqrstuvwxyz ";
const TARGET_STR: &str = "hello world";
const POPULATION_SIZE: usize = 48;
const NUM_FIT_TO_KEEP: usize = 5;
const NUM_COLUMNS: usize = 4;
const MUTATION_PROB: f64 = 0.15;

#[derive(Clone, Debug)]
struct Candidate {
    text: String,
    fitness: isize,
    in_focus: bool,
}

impl Candidate {
    fn new(text: String) -> Self {
        Self {
            text,
            fitness: -1,
            in_focus: false,
        }
    }

    fn display_str(&self, target_str: &str) -> String {
        let prefix = if self.in_focus { "➤ " } else { "  " };
        if self.fitness < 0 {
            return format!("{}{}", prefix, self.text);
        }
        let mut out = prefix.to_string();
        for (char, target_char) in self.text.chars().zip(target_str.chars()) {
            if char != target_char {
                out.push_str(&format!("\x1b[91m{}\x1b[0m", char));
            } else {
                out.push_str(&format!("\x1b[92m{}\x1b[0m", char));
            }
        }
        out
    }

    fn set_fitness(&mut self, target_str: &str) {
        self.fitness = self
            .text
            .chars()
            .zip(target_str.chars())
            .filter(|(c, t)| c == t)
            .count() as isize;
    }
}

fn reset_focus(population: &mut Vec<Candidate>) {
    for candidate in population.iter_mut() {
        candidate.in_focus = false;
    }
}

fn breed(parent_a: &Candidate, parent_b: &Candidate, mutation_prob: f64) -> Candidate {
    let mut rng = rand::thread_rng();
    let text: String = parent_a
        .text
        .chars()
        .zip(parent_b.text.chars())
        .map(|(char_a, char_b)| {
            if rng.gen_bool(mutation_prob) {
                LETTERS.chars().choose(&mut rng).unwrap()
            } else if rng.gen_bool(0.5) {
                char_a
            } else {
                char_b
            }
        })
        .collect();
    Candidate::new(text)
}

struct GeneticAlgorithm<'a, F>
where
    F: Fn(&Vec<Candidate>, &str) + 'a,
{
    population: &'a mut Vec<Candidate>,
    target_str: &'a str,
    state: usize,
    num_fit_to_keep: usize,
    population_size: usize,
    mutation_prob: f64,
    callback: F,
}

impl<'a, F> GeneticAlgorithm<'a, F>
where
    F: Fn(&Vec<Candidate>, &str) + 'a,
{
    fn new(
        population: &'a mut Vec<Candidate>,
        target_str: &'a str,
        num_fit_to_keep: usize,
        population_size: usize,
        mutation_prob: f64,
        callback: F,
    ) -> Self {
        Self {
            population,
            target_str,
            state: 0,
            num_fit_to_keep,
            population_size,
            mutation_prob,
            callback,
        }
    }
}

impl<'a, F> Iterator for GeneticAlgorithm<'a, F>
where
    F: Fn(&Vec<Candidate>, &str),
{
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        reset_focus(self.population);
        match self.state {
            0 => {
                if seed_population(self.population, self.population_size, self.target_str.len()) {
                    (self.callback)(self.population, "Seeding the population");
                    return Some(());
                } else {
                    self.state = 1;
                }
            }
            1 => {
                if compute_fitness(self.population, self.target_str) {
                    (self.callback)(self.population, "Computing fitness");
                    return Some(());
                } else {
                    // println!("done!");
                    // for c in self.population.iter() {
                    //     println!("self {} target {} result {}", c.text, self.target_str, c.fitness)
                    // }
                    self.state = 2;
                }
            }
            2 => {
                if reorder_by_fitness(self.population) {
                    (self.callback)(self.population, "Sorting by fitness");
                    return Some(());
                } else {
                    self.state = 3;
                }
            }
            3 => {
                if remove_unfit(self.population, self.num_fit_to_keep) {
                    (self.callback)(self.population, "Removing unfit candidates");
                    return Some(());
                }
                self.state = 4;
            }
            4 => {
                if breed_new(self.population, self.population_size, self.mutation_prob) {
                    (self.callback)(self.population, "Breeding new candidates");
                    return Some(());
                }
                self.state = 0;
            }
            _ => {}
        }
        None
    }
}

fn seed_population(
    population: &mut Vec<Candidate>,
    population_size: usize,
    target_str_len: usize,
) -> bool {
    if population.len() < population_size {
        population.push(Candidate::new(
            (0..target_str_len)
                .map(|_| LETTERS.chars().choose(&mut rand::thread_rng()).unwrap())
                .collect(),
        ));
        population.last_mut().unwrap().in_focus = true;
        true
    } else {
        false
    }
}

fn compute_fitness<'a>(population: &'a mut Vec<Candidate>, target_str: &'a str) -> bool {
    if let Some(ref mut candidate) = population.iter_mut().find(|c| c.fitness < 0) {
        candidate.set_fitness(target_str);
        candidate.in_focus = true;
        true
    } else {
        false
    }
}

fn reorder_by_fitness(population: &mut Vec<Candidate>) -> bool {
    let _made_swap = false;

    let n = population.len();
    for i in 0..n {
        for j in 0..n - i - 1 {
            if population[j].fitness < population[j + 1].fitness {
                population.swap(j, j + 1);
                return true;
            }
        }
    }
    false

    // for i in (0..population.len() - 1).step_by(2) {
    //     if population[i].fitness < population[i + 1].fitness {
    //         population.swap(i, i + 1);
    //         made_swap = true;
    //     }
    //     println!("first {:?} second  {:?} swap is {:?}", population[i], population[i+1], made_swap);
    // }
    // made_swap
}

fn remove_unfit(population: &mut Vec<Candidate>, num_fit_to_keep: usize) -> bool {
    if population.len() > num_fit_to_keep {
        population.pop();
        if let Some(last) = population.last_mut() {
            last.in_focus = true;
        }
        true
    } else {
        false
    }
}

fn breed_new(population: &mut Vec<Candidate>, population_size: usize, mutation_prob: f64) -> bool {
    let num_fit = population.len();
    if population.len() < population_size {
        let i = rand::thread_rng().gen_range(0..num_fit);
        let j = (i + rand::thread_rng().gen_range(1..num_fit)) % num_fit;

        reset_focus(population);

        let parent_a = population[i].clone();
        let parent_b = population[j].clone();
        population[i].in_focus = true;
        population[j].in_focus = true;

        let child = breed(&parent_a, &parent_b, mutation_prob);
        population.push(child);

        if let Some(last) = population.last_mut() {
            last.in_focus = true;
        }
        true
    } else {
        false
    }
}

fn center_text(text: &str, width: usize) -> String {
    if text.len() >= width {
        text.to_string()
    } else {
        let padding = width - text.len();
        let pad_left = padding / 2;
        let pad_right = padding - pad_left;
        format!("{}{}{}", " ".repeat(pad_left), text, " ".repeat(pad_right))
    }
}

fn display(population: &Vec<Candidate>, label: &str, column_width: usize, target_str: &str) {
    println!("\n\n");
    println!(
        "\x1b[1m\x1b[96m{}\x1b[0m\n",
        center_text(label, column_width * NUM_COLUMNS)
    );
    let num_rows = POPULATION_SIZE / NUM_COLUMNS;
    let mut cells = vec![vec![String::new(); NUM_COLUMNS]; num_rows];

    for i in 0..POPULATION_SIZE {
        let row_idx = i % num_rows;
        let col_idx = i / num_rows;

        if i >= population.len() {
            cells[row_idx][col_idx] = " ".repeat(column_width);
            continue;
        }

        let padding = column_width - target_str.len() - 2;
        cells[row_idx][col_idx] = format!(
            "{}{}",
            population[i].display_str(target_str),
            " ".repeat(padding)
        );
    }

    for row in cells {
        println!("   {}", row.join(""));
    }

    println!("\n");
}

fn main() {
    let target_str_len = TARGET_STR.len();
    let column_width = target_str_len + 6;
    let mut population: Vec<Candidate> = Vec::new();

    let display_callback = move |population: &Vec<Candidate>, label: &str| {
        // sleep(Duration::from_millis(16));
        print!("\x1b[H\x1b[J");
        display(population, label, column_width, TARGET_STR);
    };

    let mut genetic_algorithm = GeneticAlgorithm::new(
        &mut population,
        TARGET_STR,
        NUM_FIT_TO_KEEP,
        POPULATION_SIZE,
        MUTATION_PROB,
        display_callback,
    );

    loop {
        for _ in genetic_algorithm.by_ref() {}
    }
}
