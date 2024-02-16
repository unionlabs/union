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
    for i in 0..c_str.len() {
        let mut cci: Vec<Fr> = Vec::new();
        for j in 0..c_str[i].len() {
            let b: Fr = Fr::from_str(c_str[i][j]).unwrap();
            cci.push(b);
        }
        c.push(cci);
    }
    let mut m: Vec<Vec<Vec<Fr>>> = Vec::new();
    for i in 0..m_str.len() {
        let mut mi: Vec<Vec<Fr>> = Vec::new();
        for j in 0..m_str[i].len() {
            let mut mij: Vec<Fr> = Vec::new();
            for k in 0..m_str[i][j].len() {
                let b: Fr = Fr::from_str(m_str[i][j][k]).unwrap();
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
impl Poseidon {
    pub fn new() -> Poseidon {
        Poseidon {
            constants: load_constants(),
        }
    }
    pub fn ark(&self, state: &mut Vec<Fr>, c: &Vec<Fr>, it: usize) {
        for i in 0..state.len() {
            state[i].add_assign(&c[it + i]);
        }
    }

    pub fn sbox(&self, n_rounds_f: usize, n_rounds_p: usize, state: &mut Vec<Fr>, i: usize) {
        if i < n_rounds_f / 2 || i >= n_rounds_f / 2 + n_rounds_p {
            for j in 0..state.len() {
                let aux = state[j];
                state[j].square();
                state[j].square();
                state[j].mul_assign(&aux);
            }
        } else {
            let aux = state[0];
            state[0].square();
            state[0].square();
            state[0].mul_assign(&aux);
        }
    }

    pub fn mix(&self, state: &Vec<Fr>, m: &Vec<Vec<Fr>>) -> Vec<Fr> {
        let mut new_state: Vec<Fr> = Vec::new();
        for i in 0..state.len() {
            new_state.push(Fr::zero());
            for j in 0..state.len() {
                let mut mij = m[i][j];
                mij.mul_assign(&state[j]);
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
        // if inp.len() == 0 || inp.len() >= self.constants.n_rounds_p.len() - 1 {
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
        let n_rounds_f = self.constants.n_rounds_f.clone();
        let n_rounds_p = self.constants.n_rounds_p[t - 2].clone();
        for i in 0..(n_rounds_f + n_rounds_p) {
            self.ark(&mut state, &self.constants.c[t - 2], i * t);
            self.sbox(n_rounds_f, n_rounds_p, &mut state, i);
            state = self.mix(&state, &self.constants.m[t - 2]);
        }
        state
    }
}
