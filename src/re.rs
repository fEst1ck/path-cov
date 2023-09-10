#[derive(Debug, Clone)]
pub enum RegExp<T> {
	Literal(T),
	Concat(Box<RegExp<T>>, Box<RegExp<T>>),
	Alter(Box<RegExp<T>>, Box<RegExp<T>>),
	Star(Box<RegExp<T>>)
}

impl<T: Eq + Clone> RegExp<T> {
    pub fn literal(c: T) -> Self {
        Self::Literal(c)
    }

    pub fn concat(r1: RegExp<T>, r2: RegExp<T>) -> Self {
        Self::Concat(Box::new(r1), Box::new(r2))
    }

    pub fn alter(r1: Self, r2: Self) -> Self {
        Self::Alter(Box::new(r1), Box::new(r2))
    }

    pub fn star(r: Self) -> Self {
        Self::Star(Box::new(r))
    }

	pub fn parse<'a>(&self, s: &'a [T]) -> Option<(Val<T>, &'a [T])> {
		match self {
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
                let (v1, s1) = r1.parse(s)?;
                let (v2, s2) = r2.parse(s1)?;
                Some((Val::Concat(Box::new(v1), Box::new(v2)), s2))
            }
            RegExp::Alter(r1, r2 ) => {
                match r1.parse(s) {
                    Some(res) => Some(res),
                    None => r2.parse(s),
                }
            }
            RegExp::Star(r) => {
                let (vs, s1) = r.parse_star1(s);
                Some((Val::Star(vs), s1))
            }
		}
	}

    fn parse_star0<'a>(&self, s: &'a [T]) -> (Vec<Val<T>>, &'a [T]) {
        match self.parse(s) {
            Some((v, s1)) => {
                let (mut vs, s2) = self.parse_star0(s1);
                vs.push(v);
                (vs, s2)
            },
            None => (Vec::new(), s),
        }
    }

    fn parse_star1<'a>(&self, s: &'a [T]) -> (Vec<Val<T>>, &'a [T]) {
        let mut res = self.parse_star0(s);
        res.0.reverse();
        res
    }
}

pub enum Val<T> {
	Literal(T),
	Concat(Box<Val<T>>, Box<Val<T>>),
	Star(Vec<Val<T>>)
}

impl<T> Val<T> {
    pub fn reduce(self, k: usize) -> Vec<T> {
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
}

#[cfg(test)]
mod tests {
    use crate::re::*;

    #[test]
    fn test1() {
        use RegExp::*;
        // 1(21)*3
        let re = Concat(
            Box::new(Literal(1)),
            Box::new(Concat(
                Box::new(Star(
                    Box::new(Concat(Box::new(Literal(2)), Box::new(Literal(1)))))),
                Box::new(Literal(3)))
            )
        );
        let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
        let (v, _) = re.parse(&s).unwrap();
        let k = 2;
        let reduced = v.reduce(k);
        assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
    }

    #[test]
    fn test2() {
        // (12)*(13)
        let re = RegExp::concat(RegExp::star(RegExp::concat(RegExp::literal(1), RegExp::literal(2))), RegExp::concat(RegExp::literal(1), RegExp::literal(3)));
        let s = vec![1, 2, 1, 2, 1, 2, 1, 3];
        let (v, _) = re.parse(&s).unwrap();
        let k = 2;
        let reduced = v.reduce(k);
        assert!(reduced == vec![1, 2, 1, 2, 1, 3]);
    }
}
