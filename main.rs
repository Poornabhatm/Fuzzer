use rand::Rng;
use std::env;

fn main() {
    // Read input length from CLI args, or default to 16
    let args: Vec<String> = env::args().collect();
    let input_length = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(16)
    } else {
        16
    };

    // Start with one random input
    let mut best_input = generate_random_input(input_length);
    let mut best_score = mock_feedback(&best_input);

    for _ in 0..1000 {
        // Mutate the current best input
        let mutated_input = mutate_input(&best_input);
        let score = mock_feedback(&mutated_input);

        // If it's better, keep it as the new best
        if score > best_score {
            best_score = score;
            best_input = mutated_input;
        }
    }

    println!("Best Score: {}", best_score);
    println!("Best Input: {:?}", best_input);
}

// Generate a random input of a given length
fn generate_random_input(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen_range(0..=255)).collect()
}

// Mutate input by changing one random byte
fn mutate_input(input: &[u8]) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut new_input = input.to_vec();

    // Pick a random index to mutate
    if !new_input.is_empty() {
        let idx = rng.gen_range(0..new_input.len());
        new_input[idx] = rng.gen_range(0..=255); // Replace with random byte
    }

    new_input
}

// Mock feedback function: counts how many bytes are equal to 1
fn mock_feedback(input: &[u8]) -> f64 {
    input.iter().filter(|&&b| b == 1).count() as f64
}
