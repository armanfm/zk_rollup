use halo2_proofs::{
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Selector},
    poly::Rotation,
};
use pasta_curves::pallas::Base as Fp;

/// Configuração do circuito
#[derive(Clone, Debug)]
pub struct MyConfig {
    x: Column<Advice>,
    y: Column<Advice>,
    s_mul2: Selector,
}

/// Circuito que implementa a regra y = 2 * x
pub struct MyCircuit {
    pub x: Option<Fp>,
    pub y: Option<Fp>,
}

impl Circuit<Fp> for MyCircuit {
    type Config = MyConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self { x: None, y: None }
    }

    fn configure(meta: &mut ConstraintSystem<Fp>) -> Self::Config {
        let x = meta.advice_column();
        let y = meta.advice_column();
        let s_mul2 = meta.selector();

        // Cria a gate y = 2 * x
        meta.create_gate("y = 2 * x", |meta| {
            let s = meta.query_selector(s_mul2);
            let x_val = meta.query_advice(x, Rotation::cur());
            let y_val = meta.query_advice(y, Rotation::cur());
            let two_x = x_val.clone() + x_val.clone();
            vec![s * (y_val - two_x)]
        });

        MyConfig { x, y, s_mul2 }
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<Fp>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "mul2 region",
            |mut region| {
                config.s_mul2.enable(&mut region, 0)?;

                let x_val = self.x.ok_or(Error::Synthesis)?;
                let y_val = self.y.unwrap_or(x_val + x_val);

                region.assign_advice(|| "x value", config.x, 0, || Value::known(x_val))?;
                region.assign_advice(|| "y value", config.y, 0, || Value::known(y_val))?;

                Ok(())
            },
        )
    }
}
