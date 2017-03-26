use std::fmt::Write;
use std::ascii::AsciiExt;

#[derive(Debug)]
pub struct Grammar {
    pub node_types: Vec<String>,
    pub lex_rules: Vec<LexRule>,
    pub syn_rules: Vec<SynRule>,
}

#[derive(Debug)]
pub struct LexRule {
    pub ty: String,
    pub re: String,
    pub f: Option<String>,
}

#[derive(Debug)]
pub struct SynRule {
    pub name: String,
}

impl Grammar {
    pub fn generate(&self) -> String {
        let mut result = String::new();
        let has_syn_rules = !self.syn_rules.is_empty();

        result.push_str("use std::sync::{Once, ONCE_INIT};\n");
        result.push_str("use fall_tree::{NodeType, NodeTypeInfo};\n");
        result.push_str("use fall_parse::Rule;\n");
        if has_syn_rules {
            result.push_str("use fall_parse::syn;\n");
        }

        result.push_str("pub use fall_tree::{ERROR, WHITESPACE};\n\n");
        for (i, t) in self.node_types.iter().enumerate() {
            writeln!(result, "pub const {:10}: NodeType = NodeType({});", scream(t), 100 + i)
                .unwrap();
        }

        result.push_str("\npub fn register_node_types() {\n");
        result.push_str("    static REGISTER: Once = ONCE_INIT;\n");
        result.push_str("    REGISTER.call_once(||{\n");
        for t in self.node_types.iter() {
            writeln!(result, "        {}.register(NodeTypeInfo {{ name: {:?} }});", scream(t), scream(t))
                .unwrap();
        }
        result.push_str("    });\n");
        result.push_str("}\n");

        result.push_str("\npub const TOKENIZER: &'static [Rule] = &[\n");
        for &LexRule { ref ty, ref re, ref f } in self.lex_rules.iter() {
            result.push_str("    ");
            let f = match *f {
                None => "None".to_owned(),
                Some(ref f) => format!("Some({})", f)
            };
            result.push_str(&format!("Rule {{ ty: {}, re: {}, f: {} }},", scream(ty), re, f));
            result.push_str("\n");
        }
        result.push_str("];\n");

        if has_syn_rules {
            generate_syn_rules(&self.syn_rules, &mut result);
        }

        result
    }
}

fn scream(word: &str) -> String {
    word.chars().map(|c| c.to_ascii_uppercase()).collect()
}

fn generate_syn_rules(rules: &[SynRule], buff: &mut String) {
    buff.push_str("\npub const PARSER: &'static[syn::Rule] = &[\n");
    for rule in rules {
        buff.push_str("    ");
        buff.push_str(&format!(r#"syn::Rule {{ name: "{}", alts: &[] }},"#, rule.name));
        buff.push_str("\n");
    }
    buff.push_str("];\n");
}
