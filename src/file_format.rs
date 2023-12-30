//! Types related to the textual format of the CFG

type FunID = String;

type BlockID = usize;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub parse);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test0() {
		let file_format = r#"
		Function: foo
		BasicBlock: 0
		Successors: 1
		BasicBlock: 1 calls bar
		Successors: 2
		BasicBlock: 2
		Successors:

		Function: bar
		"#;
		let cfgs = parse::CFGsParser::new().parse(file_format).unwrap();
		for cfg in cfgs {
			println!("{cfg:?}");
		}
	}
}