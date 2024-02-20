extern crate ff;

pub use ff::{Field, PrimeField, PrimeFieldDecodingError};

mod constants;
mod fr;
pub use fr::*;

#[derive(Debug)]
pub struct Constants {
    pub c: Vec<Vec<Fr>>,
    pub m: Vec<Vec<Vec<Fr>>>,
    pub n_rounds_f: usize,
    pub n_rounds_p: Vec<usize>,
}
pub fn load_constants() -> Constants {
    let (c_str, m_str) = constants::constants();
    let mut c: Vec<Vec<Fr>> = Vec::new();
    for s in c_str {
        let mut cci: Vec<Fr> = Vec::new();
        for c in s {
            let b: Fr = Fr::from_str(c).unwrap();
            cci.push(b);
        }
        c.push(cci);
    }
    let mut m: Vec<Vec<Vec<Fr>>> = Vec::new();
    for i in m_str {
        let mut mi: Vec<Vec<Fr>> = Vec::new();
        for j in i {
            let mut mij: Vec<Fr> = Vec::new();
            for k in j {
                let b: Fr = Fr::from_str(k).unwrap();
                mij.push(b);
            }
            mi.push(mij);
        }
        m.push(mi);
    }
    Constants {
        c,
        m,
        n_rounds_f: 8,
        n_rounds_p: vec![
            56, 57, 56, 60, 60, 63, 64, 63, 60, 66, 60, 65, 70, 60, 64, 68,
        ],
    }
}

pub struct Poseidon {
    constants: Constants,
}

impl Default for Poseidon {
    fn default() -> Self {
        Self {
            constants: load_constants(),
        }
    }
}

impl Poseidon {
    pub fn ark(&self, state: &mut [Fr], c: &[Fr], it: usize) {
        for i in 0..state.len() {
            state[i].add_assign(&c[it + i]);
        }
    }

    pub fn sbox(&self, n_rounds_f: usize, n_rounds_p: usize, state: &mut [Fr], i: usize) {
        if i < n_rounds_f / 2 || i >= n_rounds_f / 2 + n_rounds_p {
            for s in state {
                let aux = *s;
                s.square();
                s.square();
                s.mul_assign(&aux);
            }
        } else {
            let aux = state[0];
            state[0].square();
            state[0].square();
            state[0].mul_assign(&aux);
        }
    }

    pub fn mix(&self, state: &[Fr], m: &[Vec<Fr>]) -> Vec<Fr> {
        let mut new_state: Vec<Fr> = Vec::new();
        for i in 0..state.len() {
            new_state.push(Fr::zero());
            for (j, s) in state.iter().enumerate() {
                let mut mij = m[i][j];
                mij.mul_assign(s);
                new_state[i].add_assign(&mij);
            }
        }
        new_state.clone()
    }

    pub fn hash_fixed(&self, inp: &[Fr]) -> Result<Fr, String> {
        self.hash_fixed_with_domain(inp, Fr::zero())
    }

    pub fn hash_fixed_with_domain(&self, inp: &[Fr], domain: Fr) -> Result<Fr, String> {
        let t = inp.len() + 1;
        if inp.is_empty() || inp.len() > self.constants.n_rounds_p.len() {
            return Err("Wrong inputs length".to_string());
        }
        let mut state = vec![domain];
        state.extend(inp);

        state = self.permute(state, t);
        Ok(state.remove(0))
    }

    pub fn hash_with_cap(&self, inp: &[Fr], width: usize, n_bytes: usize) -> Result<Fr, String> {
        if width < 2 {
            return Err("width must be ranged from 2 to 16".into());
        }
        if width - 2 > self.constants.n_rounds_p.len() {
            return Err(format!(
                "invalid inputs width {}, max {}",
                width,
                self.constants.n_rounds_p.len() + 1
            ));
        }

        let mut pow64 = Fr::from_str("18446744073709551616").unwrap();
        pow64.mul_assign(&Fr::from_str(&format!("{}", n_bytes)).unwrap());

        let mut state = Vec::with_capacity(width);
        state.push(pow64);
        for _ in 1..width {
            state.push(Fr::zero());
        }

        let rate = width - 1;
        {
            let mut i = 0;
            // always perform one round of permutation even when input is empty
            loop {
                // each round absorb at most `rate` elements from `inpBI`
                let mut j = 0;
                while j < rate && i < inp.len() {
                    state[j + 1].add_assign(&inp[i]);
                    i += 1;
                    j += 1;
                }
                state = self.permute(state, width);
                if i == inp.len() {
                    break;
                }
            }
        }

        Ok(state.remove(0))
    }

    fn permute(&self, mut state: Vec<Fr>, t: usize) -> Vec<Fr> {
        let n_rounds_f = self.constants.n_rounds_f;
        let n_rounds_p = self.constants.n_rounds_p[t - 2];
        for i in 0..(n_rounds_f + n_rounds_p) {
            self.ark(&mut state, &self.constants.c[t - 2], i * t);
            self.sbox(n_rounds_f, n_rounds_p, &mut state, i);
            state = self.mix(&state, &self.constants.m[t - 2]);
        }
        state
    }
}
