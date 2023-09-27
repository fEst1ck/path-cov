use std::collections::HashMap;
use std::hash::Hash;

#[derive(Debug, Clone)]
pub enum RegExp<Alphabet, Name> {
	Var(Name),
	Literal(Alphabet),
	Concat(Box<RegExp<Alphabet, Name>>, Box<RegExp<Alphabet, Name>>),
	Alter(Box<RegExp<Alphabet, Name>>, Box<RegExp<Alphabet, Name>>),
	Star(Box<RegExp<Alphabet, Name>>)
}

impl<Alphabet: Eq + Clone, Name: Eq + Clone + Hash> RegExp<Alphabet, Name> {
	pub fn var(x: Name) -> Self {
		Self::Var(x)
	}

	pub fn literal(c: Alphabet) -> Self {
		Self::Literal(c)
	}

	pub fn concat(r1: RegExp<Alphabet, Name>, r2: RegExp<Alphabet, Name>) -> Self {
		Self::Concat(Box::new(r1), Box::new(r2))
	}

	pub fn alter(r1: Self, r2: Self) -> Self {
		Self::Alter(Box::new(r1), Box::new(r2))
	}

	pub fn star(r: Self) -> Self {
		Self::Star(Box::new(r))
	}

	pub fn parse_inf<'a>(&self, s: &'a [Alphabet], env: &HashMap<Name, RegExp<Alphabet, Name>>) -> Option<(Val<Alphabet>, &'a [Alphabet])> {
		match self {
			RegExp::Var(x) => {
				let re = env.get(x).expect("name doesn't exist in env");
				re.parse_inf(s, env)
			}
			RegExp::Literal(c) => {
				if s.is_empty() {
					None
				} else {
					if c == &s[0] {
						Some((Val::Literal(c.clone()), &s[1..]))
					} else {
						None
					}
				}
			}
			RegExp::Concat(r1, r2) => {
				let (v1, s1) = r1.parse_inf(s, env)?;
				let (v2, s2) = r2.parse_inf(s1, env)?;
				Some((Val::Concat(Box::new(v1), Box::new(v2)), s2))
			}
			RegExp::Alter(r1, r2 ) => {
				match r1.parse_inf(s, env) {
					Some(res) => Some(res),
					None => r2.parse_inf(s, env),
				}
			}
			RegExp::Star(r) => {
				let (vs, s1) = r.parse_star_inf(s, env);
				Some((Val::Star(vs), s1))
			}
		}
	}

	pub fn parse_k<'a>(&self, s: &'a [Alphabet], env: &HashMap<Name, RegExp<Alphabet, Name>>, k: usize) -> Option<(Val<Alphabet>, &'a [Alphabet])> {
		match self {
			RegExp::Var(x) => {
				let re = env.get(x).expect("name doesn't exist in env");
				re.parse_k(s, env, k)
			}
			RegExp::Literal(c) => {
				if s.is_empty() {
					None
				} else {
					if c == &s[0] {
						Some((Val::Literal(c.clone()), &s[1..]))
					} else {
						None
					}
				}
			}
			RegExp::Concat(r1, r2) => {
				let (v1, s1) = r1.parse_k(s, env, k)?;
				let (v2, s2) = r2.parse_k(s1, env, k)?;
				Some((Val::Concat(Box::new(v1), Box::new(v2)), s2))
			}
			RegExp::Alter(r1, r2 ) => {
				match r1.parse_k(s, env, k) {
					Some(res) => Some(res),
					None => r2.parse_k(s, env, k),
				}
			}
			RegExp::Star(r) => {
				let (vs, s1) = r.parse_star_k(s, env, k);
				Some((Val::Star(vs), s1))
			}
		}
	}

	fn parse_star_inf<'a>(&self, mut s: &'a [Alphabet], env: &HashMap<Name, Self>) -> (Vec<Val<Alphabet>>, &'a [Alphabet]) {
		let mut acc = Vec::new();
		while let Some((val, new_s)) = self.parse_inf(s, env) {
			s = new_s;
			acc.push(val);
		}
		(acc, s)
	}

	fn parse_star_k<'a>(&self, mut s: &'a [Alphabet], env: &HashMap<Name, Self>, k: usize) -> (Vec<Val<Alphabet>>, &'a [Alphabet]) {
		let mut acc = Vec::new();
		while let Some((val, new_s)) = self.parse_k(s, env, k) {
			s = new_s;
			if acc.len() == k {
				continue;
			} else {
				acc.push(val);
			}
		}
		(acc, s)
	}
}

pub enum Val<Alphabet> {
	Literal(Alphabet),
	Concat(Box<Val<Alphabet>>, Box<Val<Alphabet>>),
	Star(Vec<Val<Alphabet>>)
}

impl<Alphabet> Val<Alphabet> {
	pub fn reduce(self, k: usize) -> Vec<Alphabet> {
		assert!(k != 0);
		match self {
			Val::Literal(c) => vec![c],
			Val::Concat(v1, v2) => {
				let mut r1 = v1.reduce(k);
				let mut r2 = v2.reduce(k);
				r1.append(&mut r2);
				r1
			},
			Val::Star(vs) => {
				let mut res = Vec::new();
				let mut counter = 0;
				for v in vs {
					if counter >= k {
						break;
					} else {
						res.append(&mut v.reduce(k))
					}
					counter += 1;
				}
				res
			},
		}
	}

	pub fn into_vec(self) -> Vec<Alphabet> {
		match self {
			Val::Literal(c) => vec![c],
			Val::Concat(v1, v2) => {
				let mut r1 = v1.into_vec();
				let mut r2 = v2.into_vec();
				r1.append(&mut r2);
				r1
			}
			Val::Star(vs) => {
				let mut res = Vec::new();
				for v in vs {
					res.append(&mut v.into_vec())
				}
				res
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::re::*;

	#[test]
	fn test1() {
		use RegExp::*;
		// 1(21)*3
		let re : RegExp<_, ()> = Concat(
			Box::new(Literal(1)),
			Box::new(Concat(
				Box::new(Star(
					Box::new(Concat(Box::new(Literal(2)), Box::new(Literal(1)))))),
				Box::new(Literal(3)))
			)
		);
		let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
		let (v, _) = re.parse_inf(&s, &HashMap::new()).unwrap();
		let k = 2;
		let reduced = v.reduce(k);
		assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
	}

	#[test]
	fn test1_() {
		use RegExp::*;
		// 1(21)*3
		let re : RegExp<_, ()> = Concat(
			Box::new(Literal(1)),
			Box::new(Concat(
				Box::new(Star(
					Box::new(Concat(Box::new(Literal(2)), Box::new(Literal(1)))))),
				Box::new(Literal(3)))
			)
		);
		let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
		let k = 2;
		let (v, _) = re.parse_k(&s, &HashMap::new(), k).unwrap();
		let reduced = v.into_vec();
		assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
	}

	#[test]
	fn test2() {
		// (12)*(13)
		let re: RegExp<_, ()> = RegExp::concat(RegExp::star(RegExp::concat(RegExp::literal(1), RegExp::literal(2))), RegExp::concat(RegExp::literal(1), RegExp::literal(3)));
		let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
		let (v, _) = re.parse_inf(&s, &HashMap::new()).unwrap();
		let k = 2;
		let reduced = v.reduce(k);
		assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
	}

	#[test]
	fn test2_() {
		// (12)*(13)
		let re: RegExp<_, ()> = RegExp::concat(RegExp::star(RegExp::concat(RegExp::literal(1), RegExp::literal(2))), RegExp::concat(RegExp::literal(1), RegExp::literal(3)));
		let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
		let k = 2;
		let (v, _) = re.parse_k(&s, &HashMap::new(), k).unwrap();
		let reduced = v.into_vec();
		assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
	}
}
