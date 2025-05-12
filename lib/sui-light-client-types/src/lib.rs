use checkpoint_summary::CheckpointSummary;
use crypto::AuthorityStrongQuorumSignInfo;

pub mod checkpoint_summary;
pub mod client_state;
pub mod committee;
pub mod consensus_state;
pub mod crypto;
pub mod digest;
pub mod header;
pub mod transaction;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Intent {
    pub scope: IntentScope,
    pub version: IntentVersion,
    pub app_id: AppId,
}
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum IntentVersion {
    V0 = 0,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum AppId {
    Sui = 0,
    Narwhal = 1,
    Consensus = 2,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(serde_repr::Serialize_repr, serde_repr::Deserialize_repr)
)]
#[repr(u8)]
pub enum IntentScope {
    TransactionData = 0,         // Used for a user signature on a transaction data.
    TransactionEffects = 1,      // Used for an authority signature on transaction effects.
    CheckpointSummary = 2,       // Used for an authority signature on a checkpoint summary.
    PersonalMessage = 3,         // Used for a user signature on a personal message.
    SenderSignedTransaction = 4, // Used for an authority signature on a user signed transaction.
    ProofOfPossession = 5, // Used as a signature representing an authority's proof of possession of its authority protocol key.
    HeaderDigest = 6,      // Used for narwhal authority signature on header digest.
    BridgeEventUnused = 7, // for bridge purposes but it's currently not included in messages.
    ConsensusBlock = 8,    // Used for consensus authority signature on block's digest.
    DiscoveryPeers = 9,    // Used for reporting peer addresses in discovery.
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IntentMessage<T> {
    pub intent: Intent,
    pub value: T,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct U64(pub u64);

#[cfg(feature = "serde")]
impl serde::Serialize for U64 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if serializer.is_human_readable() {
            self.0.to_string().serialize(serializer)
        } else {
            self.0.serialize(serializer)
        }
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for U64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            Ok(U64(String::deserialize(deserializer)?
                .parse()
                .map_err(serde::de::Error::custom)?))
        } else {
            Ok(U64(u64::deserialize(deserializer)?))
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CertifiedCheckpointSummary {
    pub data: CheckpointSummary,
    pub auth_signature: AuthorityStrongQuorumSignInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        checkpoint_summary::{CheckpointContents, CheckpointSummary},
        committee::Committee,
    };

    #[test]
    fn verify() {
        let committee: Committee = serde_json::from_str(
            r#"{"epoch":"748","voting_rights":[["gEd8ZRKR/m5oVrYZDsvpD8HkGIS/iDP6wTraSSzyj7hdzdVeCAWHyx+mTpxbJGXsBZ6sPMW/+THwSqi9lgNCdIr+AQpkTVfBkqzAvFjxQU/re9GXwrU+JTqDAyDHVjCP","150"],["gNhkGDfc1QPTb/j50fzMPmQVp8PCrSZlxd/b0Ec8nqf1N2rbMejspBhU3wi/ue2/AlgAH8g8alP6n3tNSqF9cBt5IevIYlERPvVxoekPWMWiGpLCB8NNm3iEwxKNwbgj","47"],["gPppt5z5zAin3D6xRm5BwoyzMmJFLacVDO7n8KI4NzUXtAW5w0tieB0crjeHAMuGFKbCLmQflrepjZhrgmfPBEyvcB6djInd8PtSgLXAszYCOzcMw2XyAmPBBDnf102B","45"],["gRK+r0Uf/hp3sQgfkkdkN6hqx6ABrc6PZEhPJVnfMpVFp5vLBtgbU5H3mvGZD66WElXQytyHUJ8V+5/PVeCdK7rS5cXcQXTJ8ldF4s3scsq0B18W1+2SwGHdoRX6X20D","49"],["gYiKa0xo+PSd4JbZjy0/dsMPVLjgYVQ17VqQhRc69+TcMy87XLi9sU5G2m27MEd9Dn0bTpANSvC5Q1XVJMtWDrROoreMX+m56zkGRVyeRMrvb9CV7H0vu8EVmFDXGb7C","37"],["gm/dw5PAZkeVxUTD0v0wmRkvoeoBfJXx2a2boVhBBBu6P4Qgnlc+jo0rf50PPCRHBMtZmdAXM/fV/eICWPR7Xp0M3GcpyJo5GI6izyFDlY/BJ7jgXFLjncj6mygQWUk4","37"],["hQHBIcPN7K+EOralnSu3SYHs1rgrQNpjN3YhK+g9R2uUQaXr29OgALXf1F/SJL/mBRZGZ5g7EE4MbdUMlO+v7tcvBl+qWoZMWDMdeVcxRbA/emrXtVFWzaEpYgpbyncy","43"],["hQ5p+KTRa+PSwc378Rn/BRmUIAsFZ98pDdpJ7SIegnGF5fIATuEqx1imYrGlXoloAdX7/lSSGNbLWG6mDHlHWntrkm5Q+APBZp+iiSazoLXXUpLr5aKbbeYRi32MdkO4","104"],["hSwLQqNWTjfb5UQJSQ3RCnKMD+y0Vxhb5pmL3UjsVXKN2rj9X5B7kcPyGOjRcy0ZAyN1BinG+AHszj+z7J5IUhYMo0JwHWn5mw5ar79BKo1MT+evU2o7zyJ+nAyU/8VH","85"],["hZ3c3jzHa6/DCes/kGvRXTEDo6CShn+7lcpGcqbK0LbexCFYvcWZV1r7NlFz8PsPDh9e6A/GDss/4zQbFeRkNrHvU9D8aZApTZYwtZHygX06N9XEBmQijeXC/g0WyyoL","100"],["hl0vy0MasGO6jiMVPtFaSdFgIu8MOhqxC3+mzEKaw0ZxlXTk28HFFp0r30VT4jXFCxgDpHvOi58+rHsC0pfjNCafAvhv4btWkwxoXdGqDrtlFVU4OfJGLZyM+aQdG4J0","41"],["hpz3vlLZhyxDm6+XprUikiDJ/o5fi+cKZn9jinQAc6OHvaFEOf1QNhF37CLTu6g3CvTtuzOT0bp14mx3pXj8Yei7wdBXDJ3+LrQ+anEpV/Jmx3IglwR8i4ScnlU9YTGS","154"],["iBM6LI9/6q0D6pjb+pVP99PDZlaRYKHZ5wXmr2bPeu2YTlH8V/AjVCnWU3wPCSOfCBIKqHgm6mG3owDj8WyYQf+Hv2jxUeG6STWvZcHmwLWq5fu4eZsbHs8YQh7wWg+C","83"],["iUs7gynMOxRdLQOZcSVnXSWPmH14DZgkwZ8vv2/nyUa+gGF8gEY1xWaQDDM1D4A1BsqdBiftIe5/b2s6OhzmL+DtrOarp/ms4p75QT4m2/v++NPEdOu288sCCfH1O7RM","57"],["isIXBl0R5CDQSZaui1KmmhApP+z4/UQm8CM4RzoudxaqOSCr1LRwQJf0xXHq66diElWONZJUv8bVEMq0MhC9S5gLH5Ra+ja3biOA9PZuo+WTjytYWOzY5h3vhyShH1VH","83"],["iv3O9n3JLn9M85gzBIo6NZJvuCG/DSIZeSq2KJ63UaARIbbVL5sNUfvpHmmfvR4dEMsV7ka/XmlpL5YDpurHjlTGCPnJjTzu8zIOqQgXn5HqfBkh3yWPsyX+lvUvMj4P","91"],["i0Y/KlcdUIB6H3pcsiOyw9bTff0do8PsndIEzqTwuBgVmRPfTconjcSGp1L7ZL6FACyIvgmENULvjm6i3Uziobg2PCfahghb7m2SY/JSSwWXjfgSSfphXhnaESOnbzKh","38"],["i+jOX5tClN5FLegAulXR/Mr9I/Q4Fg01ogCLHnTBqVRavS00tFklS9rhynWwF4WvEEj1cA+qmps+G9T76L99L+nnY72qsJFT3agpDYj3BQRNlengSqpa/LPGipbUkXXB","107"],["jA3PY2RS3XBhlscpWSs+kNwRn5j3kfhwftQIUB2bj2iiXnIe0bCUKY5+HTIYrSI+E02mgN0J1nR7kTi3yZAKLVfCV9ZJhpHh7KYEVF2FNlAF22T8CR+0KAUyL/B5dKXM","41"],["jDx3XWq+r8yKXS31Xw0RcEzIdeHJUL74ClxaDlex1XHfeRd6kNf71j75o8o90kfaBKTJXy3gsiu6iIt1v+arBx5nTxMWDv3CwdL/p+Tmbn5QECMFKUjJcNwFQDr2gcmL","37"],["jRw4rSwKm6E/HYLG0xjLLW+i9VOGlnYVhuKfkx27APAV1eXtl99yHSOrvilK0Vs8BwhqEFpncL063X8sOwtOvuYBzgzHF7BW6m//ylgj92+5omjZ35u4rF1m8zRU0C/S","140"],["jgLc85ODurJOxgxMNmFecnkCByws3Oc7/smojh8NmMGCo4dVmVzL4v7oSipWfC5PBUC6sy/w4ro+c5MAwpk0Jj8o3gB4WkShrvHJTvQtLgT371ZK5Uy/3CtirAA58df2","35"],["jg8NQ2cD6F1FWJVLtej2HufRkK16vqVNVdaRMOU9eXww5R8K1t+RcZ4/3kT5N9xFC3KdrH19vDfJHZF8++MRe/e/EGrb2XpdKioo6TmxN1KdxKP9Xc7IKHfllMWUl9uk","150"],["jhbu2xiJIPCqysDPHWtz3bhRNCQ6PqNASO0Z0fBYrxkj2/7S58cBibkq/oBZN/ADB+zdgjNP69ecl7ilWKU3J1zvXUClgffl2P46JEFo63K35RMTIVZCZRcIm2IK3/EU","43"],["jntr5tGjkPJV8lwgmCOSwQf7qxZ9/WqOu6t7a09ATvw2u3Wb0sfMj2RIA0hg3HEhCg9g9oXz+G3eqByQoSlGSN/kR1OD7EeASp2P9NrDviiSc9ut7RP/3KsQKOgkmj3k","82"],["jxFrxY3LvAe1JwJkZ/kCaRB4sz5Mly4kW/QTf2Ea6j+MxIvodx0SZ7hHGTNIbcD5D4PVgw0QhIZ6NMnAT1zRNFhnpGMoCSrj+PAVvERWCpPWMvJVj27NOl2QeptVoGSk","36"],["j792QbHI4yivqIeaZ8FimPAhtC9k+Z9tJeyICNQvFMjdNlkn1DgY9d2LrF6bAxFFFlXBE3LpMHh0vtn6DMlqfXVerkgNt0b2SuwmDt3f60eTCnWtIduYGEbyQSXTMKeH","111"],["j8nknwLa2uRJIaO40XTkWvmq08j8uGhPkFMtD5mFkg1uW4W98NpAl1TIH+7226G9FWgyhEfsVI75put/rDGKBMH7pGo3Tez34z58lluB5DwyfjS2wlav7XAJFCyRlYD8","119"],["kF2fS8XrwuTJWESFAUTL3bJKSr0HRRk5FWPzVzHDKxAZh0ObH2ph1T17sACVveiFFLSCuLXQ1hiOpK5DvDFAr1H4twMrnmY4RAN00YtQ5VCPxvuykk5tge8iUW+IjIjO","165"],["kKKuiotvJ4qxyR2Qe8pDJttoCsI5fiQVBXWQl+wofOEY9xUwUQPlXB/9PjVhEXVtFakoZUPULbEe1Y44w58ffjPi0H4ogkoYCt+o9Z7EiqIcYAKsQ3ahpCX3qXaHGjdw","169"],["kLRxvhxcPmD31WqMHCNeS/bMh+lL8fs20vFWuEHV5scGCZGO78aW8bs9gTHaHjhSDz2th9cLEbupx/FIpBGzyxacSkXDCZN/BNkKYWcbQqeMfUGk5nILFkhvNtMk+Nt1","150"],["kQCvDI6f4/5IO5zAIxw7mIhWcik1/36271Q/rMTbn/NFU3BQqUkatJPEG0EZ9VqjCfESkzLghBW2pqoBEXnqhsrHFS9D2Jfq7iRMXsmLX1pwMy2+3KwUzlb18Ipqaqhh","102"],["kcb0OE1hqA0fcJfKGRdnOdBSF40a0SFh0SdTmBiTPIS4YuNnhKQJy6xv3bgeyswtEKLpZCULLGNK3qKHOkNelYz8d+g4B8jRKhvpecOsyB/dBTv7Y1qkWdFxL0/xHd6B","51"],["kv2vunhTvcCZ5ulaNTpTLIkL9ZL8OH1567D6IObk2C+b+1odE9QZgQhedcs9e61kEiDJTyuVmpH9ja93lpjbHXxWh/CZQy7w7pdykoXt1CA7S+6dG2xcV3+PQEAoR2k6","37"],["k2v38Sqloaj4joAp1NvArPXeMb2uig4Yyd/yEBGir38zjpjMARHIu5VvEzzI58NQFlSjJdaB2x9q0CfoU0D9ED0gxW4PCQ8a+AQXXYbFhhr2+riDjZF7mFqR51xvYSSy","37"],["k35fl1B8tHZDQd+zgXcmAry7be/Ljoj8RNjiF78nXhwcsOXEdvjUwPlxzQGKOiF9Evei4KvIxuJaktVOD/y5MBTznsQ38w46RoLWZRjB35zlD+V14u4d/lswa394o2cX","59"],["k4XJgKfgzBAH2BpV5TJVNC3kMZnXlSOd2IVqXnOa4U8alJDteqMJFKrCWB8w/uqUBrbftOiPyd22bd5xqiRmbLOrNX8V12SeRqMcU/FElMMF3oWYCampQKWmdk+h/6Iw","190"],["k4hCkGVq8qm0cCsdSIi8BjReb2aj0A8tRriyvXgrl9YiapTJWV02Xw8ij+zNNS5XFr1ZLrqsBBEBqdfRQxU3/AjmNrELtyPeXK86d2iqbGMGqgDjQvGIMhms6Z8u54yj","45"],["lHO2S8bIwDOZG0wb15ZcJ41cz/OHncllxax5EZhDGKhE5b8huESlkTV9w3dz+I/UCGfPCQ3wfoEBR0QVcUZ2ge/uewnwMT8GSzDNTDy4K8C2f7B9jDo9WBMywaXCMwJA","76"],["lN8gjhk/uZU8OCwW8dnZ++7zBNcyi1XI/nFMgo3OkfkdplBmmK3WkDYQBLK40bcTF2Uqax+OiiAjCHR2aMnB4x6tXbP9pziHRtOVl4vttWNVWcLutCgvjOmcTHRq6mlv","191"],["lRKmfqRcMrsuZei7zPYH+25TQDgKvisFL68z3whqAsJmykTzX2gZbBfYwFlBPKKwATkEZFzcLFY8AnyQaf709WO9Z+jdbBMxW4klFT1K2suHoS9E9T+wPyyJIeGojib5","49"],["la/0YtYAaz3u1kwhqUOTxeG5Zgvn2T1sneIs8aokiDoprp/ng0A/BodaHfaUMS36AFfvcs0n0e8iiakFmHre2GgUh/aTYthRV+Wp4oO2PQEBl0HssIEBZzVhYR9L/06m","37"],["lfioPXwRB09m853Zd/Bbq7dN2wNWdmSlHhxAxDhkNR0gfpAyThgNV11nGDm7xh5uDoXartW6mUdpDuiwtlWraIRghFSebjUKMGAtHQ7CWndyv2SVw41vfz3GQmvRzhE+","48"],["ljwIccgQQZFMUx+l6LbO0hgDfag4wtJk/voiLMKlHNEfVi2dE8/L5I428qabeWPWDrWG4fbvvd14rmXCqKOSy9T5hPCqHN/LEU72umjaL7qeiwmId/4yqCLCAz3IAJQF","37"],["llAK/cn9To8WgnG+jgLPFToF10s+AKK/uyItYUysYa/ICPy/akwILKjR3sjl4NT1EjFVAs34JddG/K+TMRzLx3sQafHdtyRnXenpht34re6wy+9ldHUMyC/Ar2Y5Yfzk","57"],["lp0YAv+6VDd5aQwSsTNndJ7RvPlrXWGUlQ1xJQFhm2t8GK5uJQqyRubIvHu0dTzdGY4b0NPldnEKrNWu5f+23gs3Bxj89GmTmMOSshVl85/ia/Gc+nteNCXGj3F9Zx0p","32"],["lqoAlMzKNJceoBqPBEVxQ5YFoV2kmnsdcR+gPXWqYU/Fp/upgaxV9fM8xwD+XWU6EJuJPvTX+dgd/TWqoNQm9QCxqM0uTPHOmxrLMZoPwfAPTgqYvWC4aZtZ5Y3KfFEl","156"],["lt/bI5DNmWCk6vFmyEZueTaFl/YDug4nNpbyykaqXA6T9bOVJ9utYKba3QF6eP60Gf+AphJc2fvDXnbnls0sdqDNm25VNyarONHil7wWNsh9pujLhdXkmz1IKDmKLk85","143"],["l5ex+OuRHLzXhjsHTmtxKSzK9i0/2s+n0fSqzKFeVnpRjLURnReDtEDMriGGhuotBN19CeRnqK6U/i6Y25jshJPOfYD2awD9xseg4DfG9Dv8GEjmPXJhmvmiruTJoPxr","152"],["mcs35aJ2Zn0ZVXlEDFpAU8n7bpFyE8mi9FfrDPRe0DpJu06OEy8vMTcBhj4QbMn2EzzHoXHKhRTjRwzBFfd6s+3/9qpJFJ8ON4U74uWUvLtH/sShjNqrZ1Dbk8RdBMC0","75"],["mdAQCwk24U7GH92YvCMrKLz0NFuUwrpcW4vSO3aSHJ12ytUNYRYd4o9Tm5hupdHJDlBjssdt0ZNNQElQHBINrr0EpHMRmRq6N/+lX8JWm3EhgBfRtTLZHThjbtd8ZOhd","116"],["meTHQVEnxe8CXg/T0ZQfyajfd65jYIVcuWV4IKHW2q4rxcczOmd0J2B9hvINKY/uF8JCzUzdGci8qpmfv0uiOkmC2UQcGSIYRajqXPiPotxjaDqujBlPxIas4oY2K981","104"],["oXwL5C63LwG+HeFg3/2PYQhlnZ+G0HlWJxihQYUENMnyk9MxvBBBGwntUlifmV6oA/ho6eASqxq4yQiIdht7Hehf2Nxgb1IXHJmh7+w5Xxo7t1TGxKLc/SAaK1wJQBv2","42"],["opuVMYrwSVfndpZ02dFPeLpM7XV12mjEpwDufaOiywbRGK4k5s4WJdkkrmChwWB/A1iOa4tV5VBuJvgThcZQ81MRPJ1QpZnIbwbEM7Da5ZMtIxM8WZk6sJDcbIdSuOI7","37"],["o2aDrlUsSAZGfvZGrMDMnvbstVvAaBInr0M5uFov+7P1GsVMTiV3yVCoUgubySzXBXuxkJKcx/cEXNZcePyQ3T31w9SFmivH8f1r2IJufTuBCJqR9xjbLXrjepoJfNps","81"],["o+XXM4yg5QAK+Nt7xJAp6g03bPztUGa9ykQRI5vC/h1o6Wy8gTO19vqGROzmXOWACGBFVx/edWGBAogzImLmXk9IQKjZ2cLoglbt1TTIgaVOvX3cMlkaM2CELdvwuYtx","27"],["o/Dc0BRcRE1GWdmmsb2ItUILgPapRKpfHbWBVIWyU6AuOj1XpdALGJKu7GPjyJE5CdaCuuK3Bib37da5GDpnVgj3VtjXIQFWgi6eocg7l4vnKFtbsd8EFMzjd0A2O96f","137"],["pAcp4TSgj667rdrWNAj5HcXbPcCns0MXpI6vLddIfJI2arg04mprsn+0mquBsTNCA396H59iArQvp7mb2tlIdtunr76T6Xx/8WnZGF1WXmpo4zsmXGb0YqFYbnIcebH+","121"],["pAv4vFrL5S+EMVw/GPyvRlWQpXDMYcy7tsfWvy10HO8Iw+26AAV7csFumIhQZaWSDm7TSlXUuOK48ley573LkZVLPuEvHYW2NBwmda28H88avvCKqlavUWYqaj286kqi","44"],["pCOWxedQF3K0WO+9c7o9nDVrUVE63UQ6gm6KVe0LnvQFjJf3l2fjHTXw/saQbFoUCqHQ9OIjpn9TiRECu+92R5mVq+6tVKMLqYlBDSEyZp6joBqtW10u9PkfnlRr397M","144"],["pMiZwbn497iRuxKahVZq1VZf2KMpculWtA2LmfgCvTJBw+aDuneOe7iXeOfhqeSLEvKSzLwHUX74Gmr4Aw9IltfYsRYl+S/kx7+F4FDf4ls/LH0ZEZj5VevGK8J1xVBi","180"],["pXxoUv1Gai9KW6LMmjFMsKS42iV4JfiMSQTyVTEA3EPWsvMUXQXxZDIYSwt65XZeEAdw92JYBXSXarJApNVBwFCCbXyBb5OX68XnTqYpP31g7jDaj3ZWGriNfZJ6Hk44","47"],["pb0G/X26Ihz+1MqLQP9xp4OZ8gozB0JmaUX+1I1hkHtYeAkDmWY3UMLtoDEQEV0TAlQjgq1g5sCLvIEiT2bbZ7JkosEYawgK/1lFwb7t66ta8qzvDV5EzLuApocjFa0e","52"],["pnZUgUY9085ZjxV6jm3D5V7AY+KV20yqZBz93NGBg1EfcNZLNsluP2A4WmxbyBTdC1ZswmLlz8SMwkrsG7UFnb2m/Sf6m3EKCo0jnYaS0tyiyqtKAgy5kiBY2Ar7rgMr","278"],["ptA3fMujGqG0DIF5O5mLcY8Oo4vDgtyur0WiuAgWuUtxSVkPWiDwVa+JAKs0l+ucASNqIUFZleea/5sEfevVc8end0BbYnhgAUYy02jnSuSw+02uVJiddlRnLFAHRdPY","73"],["pxI+cUynsNT/h0VM3lbcTHtzED3aezkuyipP3RMSIzV95f6OGS1LOYDeyuNfBVirDJLTEpqKcOkMrEohXQ2Yw9p/0mLY8LOabhoWiGqnr2eQkXeFc9gt88K/plvyJiMj","39"],["p10qsU/0wKga8jd36IO/cFzbzG3qa3x0T8jdn7mOraF61nG38HzNCJ1uH1UQahAiCf59JFN9I+f1y8wriUblyUCdvcvc3gF5iospbGo5dNTQ+dcSFu3Ws0fAXVn/Mzie","42"],["p4zPuTuLrXAihvo60MLk/Nplw8tmulGKzRZI/x2i83wfWU0DG8yo/+7A2NEIDgKpF77lHlUS3Sh+kP3JR0lRhfjEXlvBjGHddT01/yRPV9AwXZutP5hrqTPYkoP8cPCK","39"],["qLA/rFy9/gKpz61gO/6mCKzMQiH1lY8xViV+mi5x+IfuWAQjt6htxauTw0q2+OynEWV7xhzcXCuSujTGmUuUYCY0q9V4QyAYZ1Qt2Us/fofJr2fJRq9Y6lDs3ImiSKDv","36"],["qMJiqwZZb1SZgjbOvWhySgWhU289GxYJ0KJdSgLyAQxgQr2btOiZ9iw87VrNuiT+GcFB2rlLz9FV77N4h/ATtH0nS4eWfrfgZHKia4HrXhI/M3ltoHGxilpcf0qcRAK5","85"],["qVVlhyH6YXw07KX09Rv0aPOFV6PpxaSncgna7RO8CQ860wLZte1MU/m6OGEPR1T7AXepfKPvD1CdXLZXYqJbsnQ3vYl/yZ2yunW9Ydd1huUv0vJ5pvOE/gh3kkB5tVx9","184"],["qeed62aDyjcdcVgeM5XgRaa/pexC+AtrkucfeAet5T4zEzqBTIM9naTCAbKUESX8F4Gwg+VDRdwKjU4oEZY3heo0g0Ggqz6h30/BoRlhkMO8aErpEGiDDvhGQWvoAmOQ","108"],["qiRzRjGHN5XjXKi/Sqqp55D6W0tNXkwOQTIbXpzgP2PBJUwtwi+lnOWRoukOyu2BBjjTMgBPR5xPkZXYbJIERJQF3p1az3sl+s+oP732uU+z6a1BwsLI1dj2Ofl73DqM","48"],["q/IHtcHrGFuaV/R+e8Yzqs1ySEL/PCwOILTdJCqy9zkPnLwNjJxNUCR9AHTkfC2rDV1hReAbciKxmp0foKBO73/KROUdPIS8bhsWK43KCdN5YEtWsSFKRgO86BHFNbLJ","142"],["rGRE1N1dAXKu+8xDtJVq6qnNwuwX691W+Ie7hqqI6uP9PsVf7buSNrsuxnK2+i26EY1CrxGm2rYV2pWIlz6gmDc1yr2dAOjjg8xHontHeN2w2X770VcYKMetKam0zbWW","41"],["remuyj9YK1/hncAalIx89fLo99AtCtFF5Fx4syb2gfhktvBSDHTjfV2Y6kAl7RhsD9zGSsKxpq5CTw7OvvmOYiDuBoReetFUCyvcH6XRybZmKHNib+sJyiHO6U/tgkYm","102"],["rfWGaY2X/XGiLXfiDMHhmUkc0b13/+NIuCmm1NNi0kz/YEp46O+W43y1I7wPVJXiEYKGMbYHIxLqsRbvWhgt6vGWEREzsyoroYiSyt/+P+m6FJCoMYT/0EdPRThp2hCw","113"],["rfjIo4FPeMHyksV/DrkxdaaFo0k9PsFjAC5HXXT7o2IDdLggDvsEdtGrXJQkKFfQDsbynFrUvJRHUeD7VrLiKhAaYujXj5hAsjuDQCqPlV7iX+9slJP4g5OU7SUBk4x8","41"],["roA5+QeBDoF8KS8n4wcBfLyA4gqrGR0K5ShEcF3fbkGaB+UBtj8Mnd26kC4ELnjzDCSE+c9Gw/j8z30YqxAIfwvmLjf+2QFdOIyQqmtxoxlL6WWmTJ5sSRD0T7O6VICv","201"],["ruB/qAfQ4XKdvKRS91d9eJVFSpsqeS2f2nhGllZ06j96xR3rG9roISK3dRYmgFmpEr1pMoIkHTAhjoilScQIez7CjhslSox/+Dh6gnpXYu6O9Qdj1B01AdPAIqwzGWVW","51"],["rv03YLO2VS6d17UaZB2bJbsTKal2s2ajHiE31DFeHRPSGJFM0XT8UjrLfzDzdQI4DbflufDyPEnj+7YtTF6rvC2ojA7/MRzY0lFov9m6faJV5u4RaiveR0c0Er6l1Qjb","159"],["r60PWt1RSI/PTwyCSkDmfHvghxhn6eGEMZwhiknQlufrs4I8RlSYO5hU/RVnW8+GAuiwJ+aWbDzR/qMGm5+vEdoYP45PjfWmb8o+wGKhOOSKlO93TwQ23+zyaUAps4gc","47"],["sFHgHeulVw4Me69nVLIO998Z+LC5FgGgqOHWapaILsqW9oyNq8zpTcbGJNMhjrDTE8R8u/CBdEpt6VyTtymvF5KTLr2UFITvjaJDcEcta8JciPr8VTJptmhbi1xIxBx3","164"],["sIKADwnYVGi7T2itDiynaVpFzUk1UOW+0r26cZ/AAMq+DQIFcUtnf1b+3NTTNHF2GEJ+15tuiQTvGX5/9qeMKfR5CNLjerXkPEm/o/duGzSdpN3La/OdHAokkUhFNUhh","212"],["sM4xNy5mSqck47Uvq5j02Dcd4NbMpJOHMQmzJpQ53/rLCxzhoOHBVhW+bfpWaPyjD4bCxgx1DMh2Mc566BJqkOu4LiOGE3Z1k7yXEtvs/jqao1NC/s+hdhhvYkTxpRCM","37"],["sWszg8cseCuYY93DD8zRX1xLtSfRnYjq6kBNodwRfrL7nmbs2gNU5tRHZ2pki+IvC3SH/sTonviafsvaOBQfGEnrRmaZxPdllPJLJA10mbMCoTIn4aw7cqzR2WjkNGqh","98"],["sk/4mJIEdCTswTaBqwk1iT24DYZkiFt1JqSgua9Yx/9ljAOzULZL5KpF9zoDjVT6DI1uHlaS9R1hXGvfGgYVhepXIMpgGVpEcDyV6QPiZuZXN51wx1WBWdaOcgYJ3/MM","118"],["smfCI0+2d3pqb6FeMdRec3iM4Q3d7pNMk+eeYtbBoVY9XVlep37clMoJwAcv9ylyAPEkWmXPivX3XcZXZD6nB41QTPkp7lObXI1ToQoSiZAM1tBmJ2GhhEsljIYGE139","40"],["st3SAp8VjBm3ZEejdv8nyl1k20UbRgEquo1Jjj3DLp22bejiHISis95vaqEDm/oNBfRdQJ+jwmZB5Ips8l6iSpFixZQNEQLkWcLf6RzC827XgsK6pICCy8f0bs7DV0xU","159"],["suZGxE1JSKbrc8RIe5n/S17VTnv337BV5VfFNFbbiYS/ivy4aMqdqz4rZ8nq986SFDVJqEnRF/dCrzUdIO7B9hD7jWzaCXA8vMlzDGIfT74Iq6quZRwAmagZySSuVJWu","150"],["sw+joe5wuwX54f+SnRS27iawt3gSaiHUPqG94sY8qWEILlpzVhHE/Br16RDKKBgWAsYhCyaVKGejVnMVfulTv/q4S68/ipz68Q7fkuuFvo32EkA2wxBjmtavnrDeEb0I","37"],["sz7y/gLKPxoXuA6Kw8FCpmFj+fFSQSNsJTie2ho9fW3Eo2WCZGlQk/Skth1HEnYVCZWHtHU/2lyQQTSEtUFciBMvbk9J1eLDRAVPi1CMWml2hrbdRCfsOzILoM4C1umf","34"],["s6tGVGlyXrPUoM910Ngdh/sHYAAFADyV2e/LezjAax8cdw9BStbm6LlU0Pm7zjTVF25DXm8UqfbUJjeTa7Mf84HklRO0RbOwxYvWxv/8p9lJpXpvhlwfLHi/ozEAdqhY","37"],["tEYrgEBlsgYyT5I6U8KlYPOtbRdX0Eding4LjBJnPpP2t8j05gEO17a7hWrK+MnPCsPKtvxwUuhee2JJvfI0vl89B90KQ1iyzhZAbVCaInRUXnom9vtLFEGrKY5DLzCi","51"],["tEemEK46P3fCQf+WsTrm2LOlPkmJ3voCs1gB6ytQLNK7H/LGTxS3ww0U2G9NWAqXB6Uyoay7sUv5arJIDnE1dNO9CsagKKDQmxUtE9Ku7s16t+Om096s09tRVj7lH4Rw","46"],["tKPHPqpZNR49Qe5DrZG5yEI8bruilJ2QrUky1wGKT3TrAy7J7GIJMWkPvjEMAMLhEU407GDyKivgRaNCvVxBsUgeaaZwfaAl6WO3nUi24eXKqaYQqe0ikCZz4DZqUVqX","112"],["tPXVIhIm9o44a2aFVgy7hdHDyA4FJaV9au3Ei5BnhVplqlzMD+2hDXuyURrz60fKEFfO0iY28QWzz6VdpHEH4uZalSZ7FXFsRh32qUHm4QhRB+p3zjWwjo8SKje90iy3","199"],["tP0nozWnb1PrsKpt6jOdOAdLXaJ5JauCFTUQ6ftKsL+dCtwNU023IF3j4qikiNyMDY+JkknSElqld0yHcuh78Pbi529o3pNyXowPm+8N0FQO/tnwentOHR82RptRf8PI","66"],["tRsvC8cgLpW2/CzthKXkxxDrhsr6Hg2Pd0er0Hu+73mY07hIeRTLA0h0MErxCQkiETE0Rmt51j0JRaP54V+69DR/1usxBWF7S7AGxU+UmGgzdAbOMSeCFHLPeEWlUXv+","114"],["tV1z8f6fmOQNOgl3GaSDtnptGjQSfwufdJE3+btSVrchXzt1MvPe9FhJhUP6YJFFFssE1Oko7wSMIgcE3J32uPNIeh2qx5V3bu2VpeaAuxpvqrJ1PHkehDR8JZurhqbM","96"],["tbx9tgibugiPmY4+K/3/HzrlvkPepGpGqLq5Jeqyt4svswLRFceligH3GRs0UM1TA1TXkWxOuAQxbZ1HlwXH5Y4zDg7Jc443X5ykP4LFQClfS0FXW+6I7vdEA2P3lnjZ","87"],["tg5nYXgyHIfjp/2D84owIMwi81DGCcYx4KbzxyNYX1NZ1GnatenNPMsSTu8jr/GFDO8N6eDa2J30eSnP1VW3bc+ZZRlholW4G+ZfWN3OWJ0e1CTCaiGqSrcVEBitsZDG","43"],["tkYbm/p0Xhx2qdstnjoWdfCwyiwZKTRz9+W7zCv+IqzfIX1lMeRxOccA+yuh4NxBA4JbaT0RmL2vmiJuj0ky1uJgelaAJ/cVj+G9Nhcw4RLw2zf8bCHu/KjpPTmq9HtT","293"],["ttsA9UDvyD0WR5QhwH8PFk6nUMZBdoQSsgpJXGNtIdQ875wUSuH3y0mlY2u9+emaF7wFq0bTncDrRWQB+LJzDCrYdUGkC/7A/Vh1bJhmO9II96ARNaZ/Peo6XAaxxkOK","50"],["tz+5woSC/U8fBOapPecQovjMsOmacuAqdtkYfoRtykQzbpzEUSjAmxmnpJaHoVTaDBbRUMBNZ6FxtyKmD9LsvU/AsDyzu8GpyLPvr3HND/jJlf3dMUs8rom41cd+sLvD","37"],["t1MHkEVHou6Teicap7xBVvJLgR0Nr7SyIzvqSnn2I6DOA6TNjsrDINUvrrOWbnYaGEzinoNLIkhzDuQx8NLLy8/yHMbbq2kxwxD6QC66HMKsWysJMmRiMNbvTYE+o4OE","101"],["t2kdLmhYSfXaEQKG3Hjwd00BIMwsRBeeGeGxdirUZGwTJwMMw27B6vEAfo2qDM1kDUK5JiRp0tefoX/1I4M+8oc1jgE3zJ2fDspvJvSXuvV+DbK3APIhTaU+qyKfusSS","48"],["t3l/cZ8Yb7R241GvufbIaytB4liy5yWQLeqbTF3rtsJApETqN25mmpKutdXIDinSAozN75+UU1XwjrKe1BkkwoYxXr0tf0Y4spu21UblvtpR/AjNsnUpJ418DqZTL8Gj","65"],["t/1kLF9QMBkDMRHHrDogXKrP+B8ROpTYa34AF7QLUip1Ne6TF3br7cpV3IgO1OxfCGB4Kl2HxmMq1pwEbuKZHlwf2sooq+HV1r3zroNm8m85dlaVHBn4wP0mtaGizDTD","48"],["uIbO5mPWaBAgWGjP2fRF948F/TSD/iI0q/T/14PAdfo3DNIKJS46Y+1SCuVjNZjOAB3DHFbYYlUfbHgjRrJvcx7USy8rCp/BH5QGBxZLcH3/83GtlO+Z8hk+ONmpdBGb","107"],["uKsSMiR+AtokPYfXeFjrTAOqQO8OhQ7jTm8ZbKAKMhQwJfqKeHfLFB+qexx+p8ZwDOOawlib3eMc9EIyhF7DuxO6T8QE8H99rvrQCcxlZptDrzrWaO5uoifbRB5jkPEk","99"],["uPS82F4S/EAJ6ixT/mo7q6kTgTbCOcjH7PZpMb2ix1fX5UScXWV5wqFHe87x0Ef2EinW914oRl/guuqLWGTZ938TBbURT1ZL4a90036ht6e2QUtMXgSwUI0Z6L+9Y48I","49"],["ua5V5+lcb06hBSe+Oy2ZGkW5fo1uJ04l9H4wMtZ/y9YAA0DNuuD4FccxCSrDdlDwD/eU8QmKAWHPDEX7VO6Ix25QHDvn8r5U4bGLWLrPVkOq+yhfPhlySoOyf1Y2mgDm","39"]]}"#,
        ).unwrap();

        // let summary = r#"{"data":{"epoch":748,"sequence_number":139789526,"network_total_transactions":3484519633,"content_digest":"HynbstNuQcTjneH5mHn6VftLBDpx66UjGjL4Qe4zTpry","previous_digest":"4Ubs7ULPjZcD8fp9zEkTsgYqrwwq7BismNUvdAKWQDtB","epoch_rolling_gas_cost_summary":{"computationCost":"7116745300406","storageCost":"182275157323200","storageRebate":"175474615884876","nonRefundableStorageFee":"1772470867524"},"timestamp_ms":1746024634920,"checkpoint_commitments":[],"end_of_epoch_data":null,"version_specific_data":[0,1,23,43,2,0,0,0,0,0]},"auth_signature":{"epoch":748,"signature":"rJF/tCVwhToSwufylT5iztCq1kSEUAalTOOi68Pd1bXlLuWYdDc7IAO2f3iVOzj4","signers_map":[58,48,0,0,1,0,0,0,0,0,73,0,16,0,0,0,1,0,2,0,3,0,4,0,5,0,6,0,8,0,10,0,11,0,14,0,16,0,17,0,18,0,20,0,21,0,22,0,24,0,27,0,28,0,29,0,32,0,33,0,35,0,38,0,40,0,41,0,43,0,44,0,45,0,46,0,47,0,49,0,50,0,52,0,56,0,57,0,59,0,61,0,62,0,63,0,64,0,65,0,67,0,69,0,70,0,75,0,76,0,77,0,78,0,80,0,82,0,83,0,84,0,86,0,88,0,89,0,90,0,91,0,92,0,93,0,95,0,97,0,98,0,100,0,101,0,102,0,103,0,105,0,106,0,107,0,108,0,109,0,110,0,112,0]}}"#;

        let (checkpoint, _): (CertifiedCheckpointSummary, CheckpointContents) = serde_json::from_str(r#"
[{"data":{"epoch":748,"sequence_number":139789526,"network_total_transactions":3484519633,"content_digest":"HynbstNuQcTjneH5mHn6VftLBDpx66UjGjL4Qe4zTpry","previous_digest":"4Ubs7ULPjZcD8fp9zEkTsgYqrwwq7BismNUvdAKWQDtB","epoch_rolling_gas_cost_summary":{"computationCost":"7116745300406","storageCost":"182275157323200","storageRebate":"175474615884876","nonRefundableStorageFee":"1772470867524"},"timestamp_ms":1746024634920,"checkpoint_commitments":[],"end_of_epoch_data":null,"version_specific_data":[0,1,23,43,2,0,0,0,0,0]},"auth_signature":{"epoch":748,"signature":"rJF/tCVwhToSwufylT5iztCq1kSEUAalTOOi68Pd1bXlLuWYdDc7IAO2f3iVOzj4","signers_map":[58,48,0,0,1,0,0,0,0,0,73,0,16,0,0,0,1,0,2,0,3,0,4,0,5,0,6,0,8,0,10,0,11,0,14,0,16,0,17,0,18,0,20,0,21,0,22,0,24,0,27,0,28,0,29,0,32,0,33,0,35,0,38,0,40,0,41,0,43,0,44,0,45,0,46,0,47,0,49,0,50,0,52,0,56,0,57,0,59,0,61,0,62,0,63,0,64,0,65,0,67,0,69,0,70,0,75,0,76,0,77,0,78,0,80,0,82,0,83,0,84,0,86,0,88,0,89,0,90,0,91,0,92,0,93,0,95,0,97,0,98,0,100,0,101,0,102,0,103,0,105,0,106,0,107,0,108,0,109,0,110,0,112,0]}},{"V1":{"transactions":[{"transaction":"9Wno28brWExBUtTT9CUgcRTyuGcdRzaWi43dNKNKG84P","effects":"8MAWCVbQ1jrYfsigkAaKWNao4Q4bhKknqtwvZHGZRYXq"},{"transaction":"2mxfKFULoYBQt4BG5DZ4YsLynPAYb1KvEX8u4HwWKo1g","effects":"CsbeCiru6AXmb6qMMcz5Dj8WuRvx7yt7XuFWdpb1xqZe"},{"transaction":"AyZqXPptwQiEJjmEMGBzYLhcieQ4jcTPkCcfX8uCwwf4","effects":"DqLqtzt5hSBieNvyhNnG1gJKhRrfV6YX3ShxtgS5teWp"},{"transaction":"HcEjLXNpYGbsRiWVG2DS8eQkj7Wprsp7goFX5HKLXNuh","effects":"GVUe2fdZdpJFVYX9Z4cHdGVYuySQ3j8jEL4QGG8u7m4t"},{"transaction":"DZSeWrpo58TVpFAigKechsuPEW9tjfKi9DfyBpMx5fS3","effects":"7Z4hPdcTXoD9ELDVeE9CKi3uemM6WAHCbGDkjNNCEYeX"},{"transaction":"67bickwLLrbrmDbCty6V2Hfh8AnEiHxKYeg112sgbKRk","effects":"Hwv1TJo2aLE5dG7mcwjikv6zxkPtaAeueBq7neKEB9U6"},{"transaction":"7u3hPMzWTdwPdLP9xmQ5na4Fhk2tD2h4DP4CTf4RnPJC","effects":"CBxsycWEFdcA5euA33jyGSkGR4U48YiKbjHYRn7KueMZ"},{"transaction":"9qzbdvim46eY7QcrAtxzUwdFLAageSKiw2AURwsFANMS","effects":"AWBzyDxSVkq6y5kQDkqzDPGf9FAQoNfxUT7KgnoQRyL8"},{"transaction":"DzCT1ByWn9S4fAi6x3HUGDktYuBPC8khwEMrGjA9Tomo","effects":"bXvAGhGXPkzKPn2ZYxqcq1zmqD2qcfHNK45M4UFpYsC"},{"transaction":"EgVfLCwUUqxSAa4MKfTctMhUfjRuk7cMiq92XWimydoq","effects":"5GmvPTjUQddNUP1tTiwFhPQXkSCgqMd74JdakGm4QzAJ"},{"transaction":"4Yhiqd7uQxvcjEBQ9YBwcbvgURPPR15UPStKUtKajt4A","effects":"C2csiQp59EVb4LuWmXhhKcX4wD6MuwjrkwNNRbxJetgP"},{"transaction":"KoXzYmRHHhg2PfRyYMyJcAFiAJCEB7LZ7VY5x94CQxa","effects":"C86sEjt7bCYjdBwQXsq28meuvcveL35ezG8mo4B3m86g"},{"transaction":"2tDijE6qU7Xvu8ghZxyvqNxrKLtTToJJGjMqo6hn4VsR","effects":"3EokdFoDk5DtL1xJ5QLMKN57bVSLHCFVeE3Q2ecAdDKd"},{"transaction":"6cPvN1JdN3saxazhDQpDiy1FRJ79SZmpgT1nUhAGMu2U","effects":"HHqkzLP9SjbbjWrGSdxXvj1bfeE7sCs3t68LbA9Jj2my"},{"transaction":"A261rK97s3dcyDAmm5PoEykTaAgdvc2RhHYVXGU4DVpX","effects":"Fs1MQNjoSpsAcsd1sdKnzndSDvPYDdjyeFyXu1pQ74Wy"},{"transaction":"FRSeJmQapRVrEWRYqoNAS7pHZMMAq68ybWLXmGomC96f","effects":"4WBm763RgGNGyDqYmdoiGUHVYSJgP9poL5zLU3FbwC9d"},{"transaction":"Gh9GXJffXg1mGpQkCNDttW1cFTtMWUAMVt32HVcq6Jpa","effects":"J8AQoZT4P7wNyGKTRDGT8d5x3ZXomCVnD1hyBQsND3Xn"},{"transaction":"32gN9cRsNnajx3Hsn5hF2fBBD7Y6zYuq9A2r8q9fVD8Y","effects":"8hDfDBHzvgtdUoUw8jMvRNFXKwF8zk2MoELfvxqYmgxA"},{"transaction":"HawgWVjiphrUYmzXwgbyj3Djgd1TcukGYK7beayKHh4f","effects":"DNs9a12kRGwijGW3rkaimZLFs4vstNui4KgkEXdXUhsA"}],"user_signatures":[["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="],["ANwtis654lta/mVMvYH9tP0RoLM5LIs4PZbrxKdLmBvPHFWvEOtJ3Isfl0xvNLaPxWxq148O7x13y1KEimpRCAmu/A+38zgjys9fWcjLi29W5kdlk/ujgyNs30yPZXqZ7g=="],["ALjIi8GPR5rZtVrc1mI0S1bg62mT1KZA8a7sLXEesC+ngFJnjkfXmKIVsvzkUHdnofLTsFPHzluKSO5Dt0xJtAquilBF8gknYZlRxr5F+XcsYo+uMn1tAMZzbDh1eKxuAw=="],["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="],["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="],["AIWYzGyS9yaoxXzwKDCxzrfeND39jih+X6svQoxMRTvoXnB9u0XjCLh+CzQ0HyhEoLxthwR/t5Dg5hQDmuDKLQ/Y+FbBNUO6aMit18L+CTtQmNGa0z0qsCoAi1mJ7SMK0w=="],["ACFYsawTNbsipaK4svioE35ZuV0xpeJTKm48KqygplJf1UILTc+1wAnpRYCYjno0XmaCfRWKoGwC9T8511CIdgQvWF0fsZmBoBffPN9WqDJBQ32V+xieIWrFx5KHFJdfRw=="],["AKnPpFz5awLIUyBS3ZcQCHCzL18j6cae2rY1+IaU+CPP+75d6mq2rHC5+9WMuMYLT1w0R03PUlqnStiy5805Jwn6UKNjY5Q6irEn8JriuNI6tG0iu0KoA9wG1MzJWMIkyw=="],["AJp5V8sHdsMMrppF0+jvhAx6ar8fA9dHvhVARAyxne7MWm3t6sPFPd9kMeluyRRxFsSBplmKgxF4pDpZNZioew6u/A+38zgjys9fWcjLi29W5kdlk/ujgyNs30yPZXqZ7g=="],["ADHh467oA9hxEncxqS55YolMStJWLReR3CWHGluncj9OPjU7zeojxLqx1OkZjbrUIxBNOhgSg1nayW1oBw9u7QzSGTn6ZER9oIit9i+tdwB8rqmxbkQ6DM2xEo2U5+5kGg=="],["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="],["AHYIaq5hCVeUC6dBQlVdL4Eh4cPadqCpOdoEbPkFsqwG6E09pOSWiBG7H5O8wCw4/6WRC5tVeH27nJojZ1DGHwpjNNkpDXV3GUKRXF3k3Pxcxh5PLo7ybQYQkkjHzCSk7Q=="],["ACj1/kzaifPvhd2VPbQ74A5XiQq2P1pyBT7yQr7C3Uqk/mB9Tgvq/gtZzgSw4S50jldTveLNWc4xuGMq8/t6Rg4npquna8tFZcXBdd5orACYz7/4VaoVyubEJeGXtuv5Ig=="],["ALJBED+sd871tbOJCi821TwFoJOwYugpUQeQFl/wPo8O5qQgBtNI2sZ3keKmmGyOddy1lLPfHdfAqK5fR3Ae+gY2yRgeQ+zcQmlv4Plw6kwqlNXrneApsAnF44RNoqyR3Q=="],["AHkDyZlyBdY5mDxX+Md+ZCpxLsuiJIChAWdcUWHpVVVNl7AhAwwGhJohYrWooPUKWY/Y6wMF8z9v3uQnL63h3gJ6j+x/ECy+c5tWVeLOAnzkwBN0AfuWD8GGA8A5jUsXVg=="],["AMk8Y8+Q7cTEw7TVtGPZFg4eEq2R3pNlR1OLSdfx+/9z2AtPgexUv5yKroLhV9oR8dIPrCuNG6r4thq7fnP2/gZ07LNPtDxKOTNyqxlsqnqo0A7RSbu3y5xaOqe4BEHjmQ=="],["AFd3NOGaAOdLkn/wYbXs3B6bkNQuNLCdfnGF6jN2IQQ3CHaWcq11coLXsrjmmH/q5mamNpiwuEGgRyko6tefsQ0Aw/12JcYP21xPj+11jLiyG7AAGslKfQz3GGvjlgVSoQ=="],["AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=="],["AKOjhJBwQi2uX5IbbHJhSnGN38zTS+UtTDMptqj7HchmtInfo3j4AKhmTfH9y9HiNZOnY1w1aNZeeS/7PBaLngNW7XoEYabK+LRJQrrmFEIGW4md+6US/lDYW5X1oVGs1A=="]]}}]"#).unwrap();

        // committee.verify_signature(checkpoint.data, checkpoint.auth_signature);

        // panic!(
        //     "{}",
        //     serde_json::to_string(&content.user_signatures[1][0]).unwrap()
        // );
        //
        // {
        //     let CheckpointContents::V1(content) = content;
        //     let mut bytes = Vec::new();
        //     bcs::serialize_into(&mut bytes, &content.transactions[0].transaction)
        //         .expect("Message serialization should not fail");
        //     panic!(
        //         "{:?} {}",
        //         content.transactions[0].transaction,
        //         Bytes::<Base64>::new(bytes)
        //     );
        // }
    }
}
