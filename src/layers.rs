//! This was extracted from the Chapter 13 exercises and moved into the core library so it could be used in later chapters.

use std::fmt;
use std::iter::FromIterator;

use rand::distributions::Uniform;
use rulinalg::matrix::{BaseMatrix, Matrix};
use std::rc::Rc;

use crate::generate_random_vector;
use crate::tensor::{Dot, Expand, Tensor};

pub trait Layer {
    fn forward(&self, inputs: &[&Tensor]) -> Vec<Tensor>;

    fn parameters(&self) -> Vec<&Tensor> {
        vec![]
    }
}

#[derive(Debug)]
pub struct Linear {
    weights: Tensor,
    bias: Option<Tensor>,
}

impl Linear {
    pub fn new(n_inputs: usize, n_outputs: usize, bias: bool) -> Linear {
        let distribution = Uniform::new(0.0, 1.0);

        let weights = Tensor::new_const(Matrix::new(
            n_inputs,
            n_outputs,
            generate_random_vector(n_inputs * n_outputs, 0.5, 0.0, &distribution),
        ));

        let bias = if bias {
            Some(Tensor::new_const(Matrix::zeros(1, n_outputs)))
        } else {
            None
        };

        Linear { weights, bias }
    }
}

impl Layer for Linear {
    fn forward(&self, inputs: &[&Tensor]) -> Vec<Tensor> {
        let rows = inputs[0].0.borrow().data.rows();
        match &self.bias {
            None => vec![inputs[0].dot(&self.weights)],
            Some(bias) => vec![&inputs[0].dot(&self.weights) + &bias.expand(0, rows)],
        }
    }

    fn parameters(&self) -> Vec<&Tensor> {
        match &self.bias {
            None => vec![&self.weights],
            Some(bias) => vec![&self.weights, bias],
        }
    }
}

pub struct Sequential {
    layers: Vec<Box<dyn Layer>>,
}

impl fmt::Debug for Sequential {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sequential {{  }}")
    }
}

impl Sequential {
    pub fn new(layers: Vec<Box<dyn Layer>>) -> Self {
        Sequential { layers }
    }

    #[allow(dead_code)]
    fn add(&mut self, layer: Box<dyn Layer>) {
        self.layers.push(layer);
    }
}

impl Layer for Sequential {
    fn forward(&self, inputs: &[&Tensor]) -> Vec<Tensor> {
        // TODO: can this be avoided
        let mut input = Tensor(Rc::clone(&inputs[0].0));

        for layer in self.layers.iter() {
            input = layer.forward(&[&input]).remove(0);
        }

        vec![input]
    }

    fn parameters(&self) -> Vec<&Tensor> {
        self.layers
            .iter()
            .map(|l| l.parameters())
            .flat_map(|v| v.into_iter())
            .collect()
    }
}

#[derive(Debug)]
pub struct Embedding {
    pub weights: Tensor,
}

impl Embedding {
    pub fn new(vocab_size: usize, embedding_size: usize) -> Embedding {
        let distribution = Uniform::new(0.0, 1.0);
        Embedding {
            weights: Tensor::new_const(Matrix::new(
                vocab_size,
                embedding_size,
                generate_random_vector(
                    vocab_size * embedding_size,
                    1.0 / (embedding_size as f64),
                    -0.5 / (embedding_size as f64),
                    &distribution,
                ),
            )),
        }
    }

    pub fn from_weights(weights: Matrix<f64>) -> Embedding {
        Embedding {
            weights: Tensor::new_const(weights),
        }
    }
}

impl Clone for Embedding {
    fn clone(&self) -> Embedding {
        Embedding {
            weights: Tensor::new_const(self.weights.0.borrow().data.clone()),
        }
    }
}

impl Layer for Embedding {
    fn forward(&self, inputs: &[&Tensor]) -> Vec<Tensor> {
        let data = Vec::from_iter(
            inputs[0]
                .0
                .borrow()
                .data
                .row(0)
                .raw_slice()
                .iter()
                .map(|v| (*v as usize)),
        );

        vec![self.weights.index_select(data)]
    }

    fn parameters(&self) -> Vec<&Tensor> {
        vec![&self.weights]
    }
}

pub struct RNNCell {
    n_hidden: usize,
    w_ih: Linear,
    w_hh: Linear,
    w_ho: Linear,
    activation: Box<dyn Layer>,
}

impl fmt::Debug for RNNCell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RNNCell {{ n_hidden: {:?}, w_ih: {:?}, w_hh: {:?}, w_ho: {:?} }}",
            self.n_hidden, self.w_ih, self.w_hh, self.w_ho
        )
    }
}

impl RNNCell {
    pub fn new(
        n_inputs: usize,
        n_hidden: usize,
        n_outputs: usize,
        activation: Box<dyn Layer>,
    ) -> RNNCell {
        let w_ih = Linear::new(n_inputs, n_hidden, true);
        let w_hh = Linear::new(n_hidden, n_hidden, true);
        let w_ho = Linear::new(n_hidden, n_outputs, true);

        RNNCell {
            n_hidden,
            w_ih,
            w_hh,
            w_ho,
            activation,
        }
    }

    pub fn create_start_state(&self, batch_size: usize) -> Tensor {
        Tensor::new_const(Matrix::zeros(batch_size, self.n_hidden))
    }
}

impl Layer for RNNCell {
    fn forward(&self, inputs: &[&Tensor]) -> Vec<Tensor> {
        let (input, hidden) = (inputs[0], inputs[1]);

        let state_part = self.w_hh.forward(&[hidden]);
        let input_part = self.w_ih.forward(&[input]);

        let mut new_state = self
            .activation
            .forward(&[&(&input_part[0] + &state_part[0])]);
        let mut output = self.w_ho.forward(&[&new_state[0]]);

        vec![output.remove(0), new_state.remove(0)]
    }

    fn parameters(&self) -> Vec<&Tensor> {
        let mut ans = self.w_ih.parameters();
        ans.append(&mut self.w_hh.parameters());
        ans.append(&mut self.w_ho.parameters());
        ans
    }
}

#[derive(Debug)]
pub struct LSTMCell {
    xf: Linear,
    xi: Linear,
    xo: Linear,
    xc: Linear,

    hf: Linear,
    hi: Linear,
    ho: Linear,
    hc: Linear,

    w_ho: Linear,

    n_hidden: usize,
}

impl LSTMCell {
    pub fn new(n_inputs: usize, n_hidden: usize, n_outputs: usize) -> LSTMCell {
        LSTMCell {
            xf: Linear::new(n_inputs, n_hidden, true),
            xi: Linear::new(n_inputs, n_hidden, true),
            xo: Linear::new(n_inputs, n_hidden, true),
            xc: Linear::new(n_inputs, n_hidden, true),

            hf: Linear::new(n_hidden, n_hidden, false),
            hi: Linear::new(n_hidden, n_hidden, false),
            ho: Linear::new(n_hidden, n_hidden, false),
            hc: Linear::new(n_hidden, n_hidden, false),

            w_ho: Linear::new(n_hidden, n_outputs, false),

            n_hidden,
        }
    }

    pub fn create_start_state(&self, batch_size: usize) -> (Tensor, Tensor) {
        let mut h = Matrix::zeros(batch_size, self.n_hidden);
        let mut c = Matrix::zeros(batch_size, self.n_hidden);

        for i in 0..batch_size {
            h[[i, 0]] = 1.0;
            c[[i, 0]] = 1.0;
        }

        (Tensor::new_const(h), Tensor::new_const(c))
    }
}

impl Layer for LSTMCell {
    #[allow(clippy::many_single_char_names)]
    fn forward(&self, inputs: &[&Tensor]) -> Vec<Tensor> {
        let (input, prev_hidden, prev_cell) = (inputs[0], inputs[1], inputs[2]);

        let f = (&self.xf.forward(&[input])[0] + &self.hf.forward(&[prev_hidden])[0]).sigmoid();
        let i = (&self.xi.forward(&[input])[0] + &self.hi.forward(&[prev_hidden])[0]).sigmoid();
        let o = (&self.xo.forward(&[input])[0] + &self.ho.forward(&[prev_hidden])[0]).sigmoid();

        let g = (&self.xc.forward(&[input])[0] + &self.hc.forward(&[prev_hidden])[0]).tanh();

        let c = &(&f * prev_cell) + &(&i * &g);
        let h = &o * &c.tanh();

        let output = self.w_ho.forward(&[&h]).remove(0);

        vec![output, h, c]
    }

    fn parameters(&self) -> Vec<&Tensor> {
        self.xf
            .parameters()
            .into_iter()
            .chain(self.xi.parameters().into_iter())
            .chain(self.xo.parameters().into_iter())
            .chain(self.xc.parameters().into_iter())
            .chain(self.hf.parameters().into_iter())
            .chain(self.hi.parameters().into_iter())
            .chain(self.ho.parameters().into_iter())
            .chain(self.hc.parameters().into_iter())
            .chain(self.w_ho.parameters().into_iter())
            .collect()
    }
}
