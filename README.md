## 🧩 Roadmap for Full Recursive Proof in zk_rollup

This project already demonstrates a working **Halo2 proof generation and verification pipeline**.  
The next major milestone is achieving a **fully recursive proof** — a proof that *verifies another Halo2 proof inside a circuit*.  

Below is a precise checklist of what remains to reach a complete recursive setup.

---

### ✅ Current Progress
- [x] Working circuits for basic computations (`fq_circuit.rs`, `ep_circuit.rs`, etc.)  
- [x] Proof generation with `create_proof`  
- [x] Verification with `verify_proof`  
- [x] Modular design separating prover, verifier, and rollup logic (`zk_rollup.rs`, `verifier_circuit.rs`)

---

### 🚧 TODO: Steps for a Real Recursive Proof

#### 1️⃣ Prepare Inner Proof Artifacts
- [ ] Serialize the **Verifying Key (VK)** of the inner proof into field elements so it can be loaded inside a circuit.  
- [ ] Represent **public inputs** and **commitments** of the inner proof as `AssignedCell<Fr>` values.  
- [ ] Pass these serialized values into the recursive circuit as advice inputs.

#### 2️⃣ Build the In-Circuit Verifier Gadget
- [ ] Implement the **verifier logic inside Halo2**, reproducing the PLONK verification equation within a circuit.  
- [ ] Compute all relevant challenges (`β, γ, α, ζ`) inside the circuit.  
- [ ] Reconstruct the polynomial commitment equation and enforce it via constraints.  
- [ ] Check consistency between the in-circuit VK and the external VK of the inner proof.

#### 3️⃣ Embed Previous Proofs
- [ ] Allow the recursive circuit to take one or more previous proofs as input.  
- [ ] Convert each proof’s commitments and evaluations into circuit-friendly field representations.  
- [ ] Optionally, compress these proofs for efficient embedding (e.g., hash or Merkle inclusion).

#### 4️⃣ Implement Proof Aggregation
- [ ] Create an **accumulator gadget** to combine multiple proofs into a single set of commitments (MSM-based or linear combination).  
- [ ] Ensure the accumulator output can be verified by the next recursive layer.

#### 5️⃣ Final Verification Layer
- [ ] Generate a new proof from the recursive circuit.  
- [ ] Verify that proof externally using the standard Halo2 verifier (`verify_proof`).  
- [ ] This final verification closes the recursion — one proof validates all previous ones.

---

### 🧠 Notes
- Recursive verification requires both **field serialization** and **circuit-safe VK encoding** — see how Zcash’s Halo2 recursive verifier handles this.  
- Modularizing the verifier gadget as its own module (e.g., `recursive_verifier.rs`) will make it easier to reuse and test.  
- Once implemented, the project will support **true proof aggregation**, enabling zk-rollup batch verification.

