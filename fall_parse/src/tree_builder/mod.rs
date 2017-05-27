use elapsed::measure_time;

use fall_tree::{Language, NodeType, File, ERROR};
use lex::{Token, LexRule, tokenize};

mod imp;

pub struct TreeBuilder(imp::TreeBuilderImpl);

impl TreeBuilder {
    pub fn current(&self) -> Option<Token> {
        self.0.current()
    }

    pub fn bump(&mut self) {
        self.0.bump()
    }

    pub fn start(&mut self, ty: Option<NodeType>) {
        self.0.start(ty)
    }

    pub fn finish(&mut self, ty: Option<NodeType>) {
        self.0.finish(ty)
    }

    pub fn rollback(&mut self, ty: Option<NodeType>) {
        self.0.rollback(ty)
    }

    pub fn error(&mut self) {
        self.start(Some(ERROR));
        self.finish(Some(ERROR));
    }
}

pub fn parse(
    lang: Language,
    text: String,
    file_type: NodeType,
    tokenizer: &[LexRule],
    parser: &Fn(&mut TreeBuilder)
) -> File {
    let (elapsed_lex, tokens) = measure_time(|| tokenize(&text, tokenizer).collect());
    let mut builder = TreeBuilder(imp::TreeBuilderImpl::new(lang, text, file_type, tokens));
    let (elapsed_parse, _) = measure_time(|| parser(&mut builder));
    builder.0.into_file(elapsed_lex, elapsed_parse)
}


