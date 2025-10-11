use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Selector, Error},
    poly::Rotation,
    arithmetic::CurveAffine, // necessário para .coordinates()
};
use pasta_curves::{
    pallas::{Base as Fp, Scalar as Fr},
    EqAffine,
    group::{Curve, prime::PrimeCurveAffine}, // import necessário para identity() e to_affine()
};
use ff::PrimeField;

#[derive(Clone, Debug)]
pub struct AffineConfig {
    px: Column<Advice>,
    py: Column<Advice>,
    rx: Column<Advice>,
    ry: Column<Advice>,
    s_gate: Selector,
}

pub struct AffineCircuit {
    pub p: Option<EqAffine>,
    pub q: Option<EqAffine>,
    pub k: Option<Fr>,
}

impl Circuit<Fp> for AffineCircuit {
    type Config = AffineConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { p: None, q: None, k: None }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let px = meta.advice_column();
        let py = meta.advice_column();
        let rx = meta.advice_column();
        let ry = meta.advice_column();
        let s_gate = meta.selector();

        meta.create_gate("Affine add check", |meta| {
            let s = meta.query_selector(s_gate);
            let rx_val = meta.query_advice(rx, Rotation::cur());
            let ry_val = meta.query_advice(ry, Rotation::cur());
            let expected_rx = meta.query_advice(px, Rotation::cur());
            let expected_ry = meta.query_advice(py, Rotation::cur());
            vec![
                s.clone() * (rx_val - expected_rx),
                s * (ry_val - expected_ry),
            ]
        });

        AffineConfig { px, py, rx, ry, s_gate }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<Fp>) -> Result<(), Error> {
        layouter.assign_region(
            || "Affine operations",
            |mut region| {
                config.s_gate.enable(&mut region, 0)?;

                // identity() agora funciona porque importamos PrimeCurveAffine
                let p = self.p.unwrap_or(EqAffine::identity());
                let q = self.q.unwrap_or(EqAffine::identity());
                let k = self.k.unwrap_or(Fr::one());

                // Converte Fr -> Fp usando to_repr + from_raw
                let k_repr = k.to_repr();
                let k_u64 = [
                    u64::from_le_bytes(k_repr[0..8].try_into().unwrap()),
                    u64::from_le_bytes(k_repr[8..16].try_into().unwrap()),
                    u64::from_le_bytes(k_repr[16..24].try_into().unwrap()),
                    u64::from_le_bytes(k_repr[24..32].try_into().unwrap()),
                ];
                let k_fp = Fp::from_raw(k_u64);

                let k_p = p * k_fp;

                // to_affine() agora funciona porque importamos pasta_curves::group::Curve
                let r = (k_p + q).to_affine();

                // Converte Fq -> Fp para assign_advice
                let p_coords = p.coordinates().unwrap();
                let px_val = Fp::from_repr(p_coords.x().to_repr()).unwrap();
                let py_val = Fp::from_repr(p_coords.y().to_repr()).unwrap();

                let r_coords = r.coordinates().unwrap();
                let rx_val = Fp::from_repr(r_coords.x().to_repr()).unwrap();
                let ry_val = Fp::from_repr(r_coords.y().to_repr()).unwrap();

                region.assign_advice(|| "px", config.px, 0, || Value::known(px_val))?;
                region.assign_advice(|| "py", config.py, 0, || Value::known(py_val))?;
                region.assign_advice(|| "rx", config.rx, 0, || Value::known(rx_val))?;
                region.assign_advice(|| "ry", config.ry, 0, || Value::known(ry_val))?;

                Ok(())
            }
        )
    }
}
