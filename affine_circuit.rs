use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Selector, Error},
    poly::Rotation,
    arithmetic::CurveAffine,
};
use pasta_curves::vesta::{Base as Fq, Affine as VestaAffine};
use pasta_curves::group::{Curve, prime::PrimeCurveAffine}; // <-- necessário para identity()

/// Configuração do circuito
#[derive(Clone, Debug)]
pub struct AffineConfig {
    px: Column<Advice>,
    py: Column<Advice>,
    rx: Column<Advice>,
    ry: Column<Advice>,
    s_gate: Selector,
}

/// Circuito que só usa pontos Vesta (Fq)
pub struct AffineCircuit {
    pub p: Option<VestaAffine>,
    pub q: Option<VestaAffine>,
}

impl Circuit<Fq> for AffineCircuit {
    type Config = AffineConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { p: None, q: None }
    }

    fn configure(meta: &mut ConstraintSystem<Fq>) -> Self::Config {
        let px = meta.advice_column();
        let py = meta.advice_column();
        let rx = meta.advice_column();
        let ry = meta.advice_column();
        let s_gate = meta.selector();

        meta.create_gate("Affine add check (dummy)", |meta| {
            let s = meta.query_selector(s_gate);
            let rx_val = meta.query_advice(rx, Rotation::cur());
            let ry_val = meta.query_advice(ry, Rotation::cur());
            let expected_rx = meta.query_advice(px, Rotation::cur());
            let expected_ry = meta.query_advice(py, Rotation::cur());
            vec![s.clone() * (rx_val - expected_rx), s * (ry_val - expected_ry)]
        });

        AffineConfig { px, py, rx, ry, s_gate }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fq>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "Affine dummy region",
            |mut region| {
                config.s_gate.enable(&mut region, 0)?;

                let p = self.p.unwrap_or(VestaAffine::identity());
                let q = self.q.unwrap_or(VestaAffine::identity());

                // Somamos apenas os pontos p + q (dummy, sem multiplicação)
                let r = (p + q).to_affine();

                let p_coords = p.coordinates().unwrap();
                let r_coords = r.coordinates().unwrap();

                // Correção: desreferenciar os valores de Fq
                region.assign_advice(|| "px", config.px, 0, || Value::known(*p_coords.x()))?;
                region.assign_advice(|| "py", config.py, 0, || Value::known(*p_coords.y()))?;
                region.assign_advice(|| "rx", config.rx, 0, || Value::known(*r_coords.x()))?;
                region.assign_advice(|| "ry", config.ry, 0, || Value::known(*r_coords.y()))?;

                Ok(())
            },
        )
    }
}
