//! Chapter6 - Intro to Backpropagation - Building Your First DEEP Neural Network.ipynb
//!
//! https://github.com/iamtrask/Grokking-Deep-Learning/blob/master/Chapter6%20-%20Intro%20to%20Backpropagation%20-%20Building%20Your%20First%20DEEP%20Neural%20Network.ipynb

use rand::distributions::{Distribution, Standard};
use rand::{thread_rng, Rng};

use grokking_deep_learning_rs::{
    dot, matrix_matrix_dot, relu_matrix, relu_vector, relu_vector_derivative, vector_matrix_dot,
    vector_vector_multiplication, Matrix,
};

fn main() {
    println!("\nCreating a Matrix or Two in Python\n");
    creating_a_matrix_or_two();

    println!("\nLearning the whole dataset!\n");
    learning_the_whole_dataset();

    println!("\nOur First \"Deep\" Neural Network\n");
    first_deep_neural_network();

    println!("\nBackpropagation\n");
    backpropagation();
}

/// Creating a Matrix or Two

fn creating_a_matrix_or_two() {
    let streetlights = vec![
        vec![1.0, 0.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 1.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![1.0, 0.0, 1.0],
    ];

    let walk_vs_stop = vec![0.0, 1.0, 0.0, 1.0, 1.0, 0.0];

    let mut weights = vec![0.5, 0.48, -0.7];

    let input = &streetlights[0];
    let goal_prediction = walk_vs_stop[0];

    let alpha = 0.1;

    for _ in 0..20 {
        let prediction = dot(input, &weights);
        let error = (goal_prediction - prediction).powi(2);
        println!("Prediction: {}, Error: {}", prediction, error);

        let delta = prediction - goal_prediction;
        for i in 0..3 {
            weights[i] -= alpha * (input[i] * delta);
        }
    }
}

/// Learning the whole dataset!

fn learning_the_whole_dataset() {
    let streetlights = vec![
        vec![1.0, 0.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 1.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![1.0, 0.0, 1.0],
    ];

    let walk_vs_stop = vec![0.0, 1.0, 0.0, 1.0, 1.0, 0.0];

    let mut weights = vec![0.5, 0.48, -0.7];

    let alpha = 0.1;

    for i in 0..40 {
        let mut total_error = 0.0;

        for r in 0..streetlights.len() {
            let input = &streetlights[r];
            let goal_prediction = walk_vs_stop[r];

            let prediction = dot(input, &weights);
            println!("Prediction: {}", prediction);

            let error = (goal_prediction - prediction).powi(2);

            total_error += error;

            let delta = prediction - goal_prediction;
            for i in 0..3 {
                weights[i] -= alpha * (input[i] * delta);
            }
        }

        println!("Error after iteration {} = {}\n", i + 1, total_error);
    }

    println!("Learned Weights: {:?}", weights);
}

/// Our first "Deep" Neural Network

#[allow(unused_variables, unused_assignments, unused_mut)]
fn first_deep_neural_network() {
    let inputs = vec![
        vec![1.0, 0.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 1.0, 1.0],
    ];

    let outputs = vec![vec![1.0], vec![1.0], vec![0.0], vec![0.0]];

    let (alpha, hidden_size) = (0.2, 4);

    let mut weights_1: Matrix = random_matrix(3, hidden_size, &Standard);
    let mut weights_2: Matrix = random_matrix(hidden_size, 1, &Standard);

    let hidden_layer = relu_matrix(matrix_matrix_dot(&inputs, &weights_1));
    let output = matrix_matrix_dot(&hidden_layer, &weights_2);
}

/// Backpropagation

fn backpropagation() {
    let inputs = vec![
        vec![1.0, 0.0, 1.0],
        vec![0.0, 1.0, 1.0],
        vec![0.0, 0.0, 1.0],
        vec![1.0, 1.0, 1.0],
    ];

    let outputs = vec![vec![1.0], vec![1.0], vec![0.0], vec![0.0]];

    let alpha = 0.2;

    // Weight values taken from the python notebooks for reproducing results.
    let mut weights_0_1: Matrix = vec![
        vec![-0.165_955_99, 0.440_648_99, -0.999_771_25, -0.395_334_85],
        vec![-0.706_488_22, -0.815_322_81, -0.627_479_58, -0.308_878_55],
        vec![-0.206_465_05, 0.077_633_47, -0.161_610_97, 0.370_439],
    ];

    let mut weights_1_2: Matrix = vec![
        vec![-0.591_095_5],
        vec![0.756_234_87],
        vec![-0.945_224_81],
        vec![0.340_935_02],
    ];

    for it in 0..60 {
        let mut total_error = 0.0;

        for i in 0..4 {
            let hidden_layer = relu_vector(vector_matrix_dot(&inputs[i], &weights_0_1));
            let prediction = vector_matrix_dot(&hidden_layer, &weights_1_2)[0];

            let error: f64 = (prediction - outputs[i][0]).powi(2);
            total_error += error;

            let delta_2_1 = prediction - outputs[i][0];
            let delta_1_0 = vector_vector_multiplication(
                &weights_1_2.iter().map(|v| v[0] * delta_2_1).collect(),
                &relu_vector_derivative(hidden_layer.clone()),
            );

            let weight_deltas_1_2: Matrix =
                hidden_layer.iter().map(|v| vec![v * delta_2_1]).collect();

            let weight_deltas_0_1: Matrix = inputs[i]
                .iter()
                .map(|v| delta_1_0.iter().map(|v2| v * v2).collect())
                .collect();

            for i in 0..weights_1_2.len() {
                for j in 0..weights_1_2[i].len() {
                    weights_1_2[i][j] -= alpha * weight_deltas_1_2[i][j];
                }
            }

            for i in 0..weights_0_1.len() {
                for j in 0..weights_0_1[i].len() {
                    weights_0_1[i][j] -= alpha * weight_deltas_0_1[i][j];
                }
            }
        }

        if (it + 1) % 10 == 0 {
            println!("Error: {}", total_error);
        }
    }
}

fn random_matrix(rows: usize, columns: usize, dist: &impl Distribution<f64>) -> Matrix {
    (0..rows)
        .map(|_| {
            (0..columns)
                .map(|_| 2.0 * thread_rng().sample(dist) - 1.0)
                .collect()
        })
        .collect()
}
