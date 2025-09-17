use rand::Rng;
use std::env;

fn main() {
    // STEP 1: Read input length from CLI args, or default to 16
    let args: Vec<String> = env::args().collect();
    let input_length = if args.len() > 1 {
        args[1].parse::<usize>().unwrap_or(16)
    } else {
        16
    };

    let mut best_input = vec![];
    let mut best_score = f64::MIN;

    for _ in 0..1000 {
        // STEP 2: Generate a random input of given length
        let input = generate_random_input(input_length);

        // STEP 3: Evaluate mock feedback score
        let score = mock_feedback(&input);

        // STEP 4: Track the best input
        if score > best_score {
            best_score = score;
            best_input = input.clone();
        }
    }

    println!("Best Score: {}", best_score);
    println!("Best Input: {:?}", best_input);
}

// Generate random input of a given length
fn generate_random_input(length: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..length).map(|_| rng.gen_range(0..=255)).collect()
}

// Mock feedback function (scalar output)
fn mock_feedback(input: &Vec<u8>) -> f64 {
    input.iter().filter(|&&b| b == 1).count() as f64
}
