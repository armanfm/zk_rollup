## 🧩 zk_rollup: Roadmap for Full Recursive Proof

This project implements **proof aggregation** with Halo2.  

### ✅ Current Progress
- [x] Individual computation circuits (`PaiCircuit`) generate valid proofs.
- [x] `VerifierCircuit` can read external proofs and public inputs.
- [x] `AggregatorCircuit` collects multiple `VerifierCircuit`s and produces an **aggregated proof** (`proof_aggregated.bin`).
- [x] `zk_rollup.rs` handles:
  - [x] Reading proofs from files.
  - [x] Converting `Fp` inputs to `Fr`.
  - [x] Generating VK/PK for aggregator.
  - [x] Producing the final aggregated proof.
  - [x] **Private verification of the aggregated proof within the rollup** ✅

* Aggregated proofs still do not verify individual proofs inside the circuit (full recursion not implemented yet).

---

### 🚧 Next Steps for Full Recursive Proof

#### 1️⃣ Prepare Inner Proof Artifacts
- [ ] Serialize inner proof VKs into circuit-friendly field elements.
- [ ] Represent public inputs and commitments as `AssignedCell<Fr>`.

#### 2️⃣ Build In-Circuit Verifier Gadget
- [ ] Implement PLONK verification inside a Halo2 circuit.
- [ ] Compute challenges (`β, γ, α, ζ`) and reconstruct polynomial commitments.
- [ ] Enforce VK consistency between inner and aggregator proofs.

#### 3️⃣ Embed Previous Proofs
- [ ] Feed one or more prior proofs into the recursive circuit.
- [ ] Convert proofs’ commitments/evaluations into field elements.
- [ ] Optionally compress proofs for efficiency (hash/Merkle).

#### 4️⃣ Proof Aggregation
- [ ] Build an accumulator gadget to combine multiple proofs into one.
- [ ] Ensure output is verifiable by the next recursive layer.

#### 5️⃣ Final Verification Layer
- [ ] Generate a recursive proof from the aggregator circuit.
- [ ] Verify externally with standard `verify_proof`.
- [ ] Close the recursion: one proof validates all previous ones.

---

### ⚠️ Current Limitation
- Aggregated proofs **do not yet validate individual proofs inside the circuit**.  
- Implementing full recursive verification is the next milestone.


