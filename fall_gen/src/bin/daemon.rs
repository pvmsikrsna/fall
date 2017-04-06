#[macro_use]
extern crate serde_derive;
extern crate jsonrpc_core;
extern crate jsonrpc_minihttp_server;
extern crate fall_gen;
extern crate fall_tree;

use std::collections::HashSet;

use jsonrpc_core::{IoHandler, Params, to_value};
use jsonrpc_minihttp_server::{ServerBuilder};

use fall_tree::{Node, NodeType, walk_tree, AstNode};
use fall_tree::search::{child_of_type};
use fall_gen::syntax::*;
use fall_gen::ast::*;

type Spans = Vec<(u32, u32, &'static str)>;

#[derive(Serialize, Default)]
struct Response {
    spans: Spans
}

fn main() {
    let mut io = IoHandler::new();
    io.add_method("colors", |params: Params| {
        let (text, ): (String, ) = params.parse().unwrap();
        let spans = ::std::panic::catch_unwind(|| colorize(text)).unwrap_or_default();
        println!("spans = {:?}\n\n\n", spans);
        let r = to_value(Response { spans: spans }).unwrap();
        Ok(r)
    });

    let server = ServerBuilder::new(io)
        .threads(3)
        .start_http(&"127.0.0.1:9292".parse().unwrap())
        .unwrap();

    println!("Starting");
    server.wait().unwrap();
}


fn colorize(text: String) -> Spans {
    let file = fall_gen::FallFile::parse(text);
    let ast = file.ast();
    let mut result = vec![];
    colorize_tokens(ast.node(), &mut result);
    colorize_file(ast, &mut result);
    result
}

fn colorize_tokens(node: Node, spans: &mut Spans) {
    let keywords = [KW_RULE, KW_VERBATIM, KW_NODES, KW_TOKENIZER];
    walk_tree(node, |node| {
        if keywords.contains(&node.ty()) {
            colorize_node(node, "keyword", spans)
        }
        if [HASH_STRING, SIMPLE_STRING].contains(&node.ty()) {
            colorize_node(node, "string", spans)
        }
    })
}

fn colorize_node(node: Node, color: &'static str, spans: &mut Spans) {
    spans.push((node.range().start(), node.range().end(), color))
}

fn colorize_child(node: Node, child: NodeType, color: &'static str, spans: &mut Spans) {
    if let Some(child) = child_of_type(node, child) {
        colorize_node(child, color, spans);
    }
}

fn colorize_file(file: File, spans: &mut Spans) {
    let token_names: HashSet<_> = file.tokenizer_def().lex_rules().map(|r| r.node_type()).collect();
    walk_tree(file.nodes_def().node(), |n| {
        if n.ty() == IDENT {
            let color = if token_names.contains(n.text()) { "token" } else { "rule" };
            colorize_node(n, color, spans);
        }
    });

    for rule in file.tokenizer_def().lex_rules() {
        colorize_child(rule.node(), IDENT, "token", spans);
    }

    for rule in file.syn_rules() {
        colorize_child(rule.node(), IDENT, "rule", spans);
        for alt in rule.alts() {
            colorize_alt(alt, &token_names, spans)
        }
    }

    //    //    colorize_node_types(node_types);
    //    //    colorize_lex_rules(lex_rules);
}

fn colorize_alt(alt: Alt, tokens: &HashSet<&str>, spans: &mut Spans) {
    for part in alt.parts() {
        colorize_part(part, tokens, spans)
    }
}

fn colorize_part(part: Part, tokens: &HashSet<&str>, spans: &mut Spans) {
    colorize_child(part.node(), SIMPLE_STRING, "token", spans);
    if let Some(name) = part.name() {
        let color = if tokens.contains(name) { "token" } else { "rule" };
        colorize_child(part.node(), IDENT, color, spans);
    }
    if let Some((_, alts)) = part.op() {
        colorize_child(part.node(), IDENT, "builtin", spans);
        colorize_child(part.node(), LANGLE, "builtin", spans);
        colorize_child(part.node(), RANGLE, "builtin", spans);
        for alt in alts {
            colorize_alt(alt, tokens, spans)
        }
    }
}
