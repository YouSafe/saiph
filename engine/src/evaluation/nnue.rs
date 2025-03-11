use types::color::Color;
use types::piece::{PieceType, NUM_PIECES};
use types::square::{Square, NUM_SQUARES};

pub type Matrix<T, const ROWS: usize, const COLS: usize> = [[T; ROWS]; COLS];

pub type Vector<T, const ROWS: usize> = [T; ROWS];

pub struct FeatureIndex(usize);

impl FeatureIndex {
    pub fn new(color: Color, piece: PieceType, square: Square) -> Self {
        Self(
            color as usize * (NUM_PIECES * NUM_SQUARES)
                + piece as usize * NUM_SQUARES
                + square as usize,
        )
    }
}

// TODO: train network and replace with new model
static MODEL: NNUE = NNUE {
    input_layer: Layer {
        weights: [[0; NNUE::HIDDEN]; NNUE::FEATURES],
        biases: [[0; NNUE::HIDDEN]; 1],
    },
    hidden_layer: Layer {
        weights: [[0; 1]; { NNUE::HIDDEN * 2 }],
        biases: [[0; 1]; 1],
    },
};

/// Sequential Neural network with dense layers
///
/// Architecture is (768 -> 128)x2 -> 1
#[repr(C)]
pub struct NNUE {
    input_layer: Layer<{ NNUE::FEATURES }, { NNUE::HIDDEN }>,
    hidden_layer: Layer<{ NNUE::HIDDEN * 2 }, 1>,
}

pub struct Layer<const INPUTS: usize, const OUTPUTS: usize> {
    weights: Matrix<i16, OUTPUTS, INPUTS>,
    biases: Matrix<i16, OUTPUTS, 1>,
}

#[derive(Debug, Clone)]
pub struct NNUEAccumulator {
    values: Vector<i16, { NNUE::HIDDEN }>,
}

impl Default for NNUEAccumulator {
    fn default() -> Self {
        Self {
            values: MODEL.input_layer.biases[0],
        }
    }
}

impl NNUEAccumulator {
    pub fn set_feature(&mut self, index: FeatureIndex) {
        for (elem, weight) in self
            .values
            .iter_mut()
            .zip(&MODEL.input_layer.weights[index.0])
        {
            *elem += *weight;
        }
    }

    pub fn unset_feature(&mut self, index: FeatureIndex) {
        for (elem, weight) in self
            .values
            .iter_mut()
            .zip(&MODEL.input_layer.weights[index.0])
        {
            *elem -= *weight;
        }
    }
}

impl NNUE {
    const FEATURES: usize = 768;
    const HIDDEN: usize = 128;

    const SCALE: i32 = 400;
    const QA: i32 = 255;
    const QB: i32 = 255;

    pub fn evaluate(stm: &NNUEAccumulator, nstm: &NNUEAccumulator) -> i32 {
        let mut output: i32 = 0;

        let (stm_weights, nstm_weights) = MODEL.hidden_layer.weights.split_at(Self::HIDDEN);

        output += stm
            .values
            .iter()
            .zip(stm_weights)
            .chain(nstm.values.iter().zip(nstm_weights))
            .map(|(&input, &[weight])| i32::from(input).clamp(0, Self::QA) * i32::from(weight))
            .sum::<i32>();

        output += MODEL.hidden_layer.biases[0][0] as i32;

        output *= Self::SCALE;
        output /= Self::QA * Self::QB;

        output
    }
}
