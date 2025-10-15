


pub mod affine_circuit;
pub use affine_circuit::AffineCircuit;

pub mod aggregator_circuit;
pub use aggregator_circuit::AggregatorCircuit;

pub mod fq_circuit;
pub use fq_circuit::{MyFq, FqOp}; 


pub mod my_fq_circuit;
pub use my_fq_circuit::MyFqCircuit;




pub mod recursive_circuit;
pub mod layouter_circuit;

// Exporta os circuitos com nomes claros para evitar conflito
pub use recursive_circuit::RecursiveCircuit as RecursiveCircuitBase;
pub use layouter_circuit::RecursiveCircuit as RecursiveCircuitLayouter;