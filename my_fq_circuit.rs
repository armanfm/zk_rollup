use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Circuit, Column, Advice, Selector, ConstraintSystem, Error},
    poly::Rotation,
};
use pasta_curves::vesta::Base as Fq;

#[derive(Clone, Debug)]
pub struct MyFqConfig {
    pub x_fq: Column<Advice>,
    pub y_fq: Column<Advice>,
    pub s_mul2_fq: Selector,
}

pub struct MyFqCircuit {
    pub x_fq: Option<Fq>,
    pub y_fq: Option<Fq>,
}

impl Circuit<Fq> for MyFqCircuit {
    type Config = MyFqConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { x_fq: None, y_fq: None }
    }

    fn configure(meta: &mut ConstraintSystem<Fq>) -> Self::Config {
        let x_fq = meta.advice_column();
        let y_fq = meta.advice_column();
        let s_mul2_fq = meta.selector();

        meta.create_gate("y = 2 * x (Fq)", |meta| {
            let s = meta.query_selector(s_mul2_fq);
            let x_val = meta.query_advice(x_fq, Rotation::cur());
            let y_val = meta.query_advice(y_fq, Rotation::cur());
            vec![s * (y_val - (x_val.clone() + x_val.clone()))]
        });

        MyFqConfig { x_fq, y_fq, s_mul2_fq }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fq>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "mul2 region Fq",
            |mut region| {
                config.s_mul2_fq.enable(&mut region, 0)?;

                let x_val = self.x_fq.ok_or(Error::Synthesis)?;
                let y_val = self.y_fq.unwrap_or(x_val + x_val);

                region.assign_advice(|| "x value Fq", config.x_fq, 0, || Value::known(x_val))?;
                region.assign_advice(|| "y value Fq", config.y_fq, 0, || Value::known(y_val))?;

                Ok(())
            },
        )
    }
}
