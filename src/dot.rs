//! Dot file parsing and generation.
//!
//! Only a subset of the dot language is supported. Specifically,
//!
//! * graph IDs must be alphanumerical string or must be quoted (without escaping),
//! * Vertex IDs must be non-negative integers,
//! * subgraphs are not supported,
//! * attributes (in any form) are not supported,
//! * ports and compass points are not supported.
//!
use crate::sparse::{directed, undirected};
use crate::{DirectedGraph, Graph, UndirectedGraph};

use combine::StreamOnce;
use combine::error::StreamError;
use combine::parser::char::{alpha_num, char, digit, letter, spaces, string};
use combine::parser::error::Silent;
use combine::parser::sequence::Skip;
use combine::parser::{Parser, *};
use combine::stream::position::{DefaultPositioned, Positioner};
use combine::stream::{ResetStream, Stream, easy, position};
use combine::{EasyParser, ParseError, attempt, between, choice, many, many1, optional, satisfy};
use itertools::Itertools;

/* ************************************************************************** */

pub fn directed_from_dot<I, X, S>(dot: I)
    -> Result<directed::AdjacencySets, easy::ParseError<position::Stream<I, X>>>
where
    S: StreamError<char, I::Range>,
    X: Positioner<char>,
    X::Position: std::fmt::Display + Default,
    I: ResetStream,
    I: StreamOnce<Token = char>,
    I: DefaultPositioned<Positioner = X>,
    I::Range: PartialEq + std::fmt::Display,
    I::Error: ParseError<char, I::Range, I::Position, StreamError = S>,
    I::Error: ParseError<char, I::Range, X::Position, StreamError = S>,
{
    let input = position::Stream::new(dot);
    match strict_directed_graph().easy_parse(input) {
        Ok((graph, _)) => Ok(graph),
        Err(err) => {
            Err(err)
        }
    }
}

pub fn undirected_from_dot<I, X, S>(dot: I)
    -> Result<undirected::AdjacencySets, easy::ParseError<position::Stream<I, X>>>
where
    S: StreamError<char, I::Range>,
    X: Positioner<char>,
    X::Position: std::fmt::Display + Default,
    I: ResetStream,
    I: StreamOnce<Token = char>,
    I: DefaultPositioned<Positioner = X>,
    I::Range: PartialEq + std::fmt::Display,
    I::Error: ParseError<char, I::Range, I::Position, StreamError = S>,
    I::Error: ParseError<char, I::Range, X::Position, StreamError = S>,
{
    let input = position::Stream::new(dot);
    match strict_undirected_graph().easy_parse(input) {
        Ok((graph, _)) => Ok(graph),
        Err(err) => {
            Err(err)
        }
    }
}

/* ************************************************************************** */
/* Utils */

fn tok<I,O, P>(p: P) -> Skip<P, Silent<impl Parser<I, Output = ()>>>
where
    I: Stream<Token = char>,
    P: Parser<I, Output = O>,
{
    // TODO also handle comments
    p.skip(spaces().silent())
}

/* ************************************************************************** */
/* Graph */

fn require_strict<I>() -> impl Parser<I, Output = ()>
where I: Stream<Token = char>
{
    tok(string("strict")).map(|_| ())
}

fn strict_directed_graph<I, G: DirectedGraph>() -> impl Parser<I, Output = G>
where I: Stream<Token = char>
{
    spaces().silent().with(
        require_strict()
        .with(
            tok(string("digraph"))
            .with(strict_graph())
        )
    )
}

fn strict_undirected_graph<I, G: UndirectedGraph>() -> impl Parser<I, Output = G>
where I: Stream<Token = char>
{
    spaces().silent().with(
        require_strict()
        .with(
            tok(string("graph"))
            .with(strict_graph())
        )
    )
}

/// # Invariant
///
/// The `is_directed` parameter must be consistent with the type of `G`.
///
fn strict_graph<I, G>() -> impl Parser<I, Output = G>
where
    I: Stream<Token = char>,
    G: Graph,
{
    let is_directed = G::IS_DIRECTED;
    tok(optional(graph_id()))
        .skip(tok(char('{')))
        .and(tok(stmt_list(is_directed)))
        .skip(tok(char('}')))
        .map(move |(_gid, stmts)| {
            let order = stmts.iter().filter_map(|stmt| 
                if let Stmt::Node(n) = stmt { Some(*n + 1) } else { None }
            ).max().unwrap_or(0);
            let edges = stmts
                .into_iter()
                .filter_map(|stmt|
                    if let Stmt::Edge(edges) = stmt {
                        Some(edges)
                    } else {
                        None
                    }
                )
                .flatten();
            G::from_edges_with_order(order, edges)
        })
}

/* ************************************************************************** */
/* Statements */

enum Stmt {
    Node(usize),
    Edge(Vec<(usize, usize)>),
}

fn stmt_list<I>(is_directed: bool) -> impl Parser<I, Output = Vec<Stmt>>
where I: Stream<Token = char>
{
    many(stmt(is_directed))
    .message("statement list")
}

fn stmt<I>(is_directed: bool) -> impl Parser<I, Output = Stmt>
where I: Stream<Token = char>
{
    // It would be more efficient (LL(1)) to first parse the first token
    // (node_id or subgraph) and then dispatch.
    choice((
        attempt(edge_stmt(is_directed).map(Stmt::Edge)),
        attempt(node_stmt().map(Stmt::Node)),
    ))
    .message("statement")
}

/* ************************************************************************** */
/* Nodes */

fn node_stmt<I>() -> impl Parser<I, Output = usize>
where I: Stream<Token = char>
{
    node_id_numeral()
    .skip(optional(tok(char(';'))))
    .message("node statement")
}

/* ************************************************************************** */
/* Edges */

fn edge_op<I>(is_directed: bool) -> impl Parser<I, Output = ()>
where I: Stream<Token = char>
{
    tok(if is_directed { string("->") } else { string("--") }).map(|_| ())
    .message("edge operator")
}

fn edge_stmt<I>(is_directed: bool) -> impl Parser<I, Output = Vec<(usize, usize)>>
where I: Stream<Token = char>
{
    many1(
        attempt(node_id().skip(edge_op(is_directed)))
    )
    .and(node_id())
    .skip(optional(tok(char(';'))))
    .map(|(nodes, last): (Vec<usize>, usize)|
        nodes.into_iter().chain(std::iter::once(last)).tuple_windows().collect()
    )
    .message("edge_stmt")
}

/* ************************************************************************** */
/* IDs */

// TODO: handle multiline strings and '+' concatenation

fn graph_id<I>() -> impl Parser<I, Output = String>
where I: Stream<Token = char>
{
    choice((
        tok(graph_id_quoted()),
        tok(graph_id_string()),
    ))
    .message("graph_id")
}

fn graph_id_string<I>() -> impl Parser<I, Output = String> 
where I: Stream<Token = char>
{
    letter()
        .and(many(alpha_num()))
        .map(|(first, rest): (char, String)| {
            first.to_string() + &rest
        })
}

fn graph_id_quoted<I>() -> impl Parser<I, Output = String>
where I: Stream<Token = char>
{
    between(
        char('"'),
        char('"'),
        many1(satisfy(|c| c != '"')),
    )
}

fn node_id<I>() -> impl Parser<I, Output = usize>
where I: Stream<Token = char>
{
    tok(node_id_numeral())
    .message("node_id")
}

fn node_id_numeral<I>() -> impl Parser<I, Output = usize>
where I: Stream<Token = char>
{
    many1(digit())
    .map(|digits: String| {
        digits.parse::<usize>().unwrap()
    })
    .message("node_id_numeral")
}

/* ************************************************************************** */
/* Printer */

pub trait ToDot {
    fn to_dot_named<W: std::fmt::Write>(&self, graph_id: &str, writer: &mut W) -> std::fmt::Result;
    fn to_dot<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        self.to_dot_named("", writer)
    }
}

fn escape_quotes(s: &str) -> String {
    s.replace('\"', "\\\"")
}

impl<G: Graph> ToDot for G
{
    fn to_dot_named<W: std::fmt::Write>(
        &self,
        graph_id: &str,
        writer: &mut W
    ) -> std::fmt::Result
    {
        let graph_id = if graph_id.is_empty() {
            "".into()
        } else {
            format!(r#""{}""#, escape_quotes(graph_id))
        };
        let typ = if G::IS_DIRECTED { "digraph" } else { "graph" };
        let edge_op = if G::IS_DIRECTED { "->" } else { "--" };

        writeln!(writer, "strict {} {} {{", typ, graph_id)?;

        for v in 0..self.order() {
            if self.adjacents(v).next().is_none() {
                writeln!(writer, "    {};", v)?;
            }
        }
        for (v, w) in self.edges() {
            writeln!(writer, "    {} {} {};", v, edge_op, w)?;
        }
        writeln!(writer, "}}")
    }
}

/* ************************************************************************** */
/* Tests */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::RandomGraph;

    #[test]
    fn test_directed_from_dot() {
        let dot = r#"
            strict digraph {
                0 -> 1;
                1 -> 2 -> 0;
                2 -> 0;
                3;
            }
        "#;
        let graph = directed_from_dot(dot).unwrap();
        assert_eq!(graph.order(), 4);
        assert!(graph.is_edge(0, 1));
        assert!(graph.is_edge(1, 2));
        assert!(graph.is_edge(2, 0));

        assert!(!graph.is_edge(0, 0));
        assert!(!graph.is_edge(0, 2));
        assert!(!graph.is_edge(0, 3));
        assert!(!graph.is_edge(1, 0));
        assert!(!graph.is_edge(1, 1));
        assert!(!graph.is_edge(1, 3));
        assert!(!graph.is_edge(2, 1));
        assert!(!graph.is_edge(2, 2));
        assert!(!graph.is_edge(2, 3));
        assert!(!graph.is_edge(3, 0));
        assert!(!graph.is_edge(3, 1));
        assert!(!graph.is_edge(3, 2));
        assert!(!graph.is_edge(3, 3));
    }

    #[test]
    fn test_undirected_from_dot() {
        let dot = r#"
            strict graph {
                0 -- 1;
                1 -- 2 -- 0;
                2 -- 0;
                3;
            }
        "#;
        let graph = undirected_from_dot(dot).unwrap();
        assert_eq!(graph.order(), 4);
        assert!(graph.is_edge(0, 1));
        assert!(graph.is_edge(0, 2));
        assert!(graph.is_edge(1, 0));
        assert!(graph.is_edge(1, 2));
        assert!(graph.is_edge(2, 0));
        assert!(graph.is_edge(2, 1));

        assert!(!graph.is_edge(0, 0));
        assert!(!graph.is_edge(0, 3));
        assert!(!graph.is_edge(1, 1));
        assert!(!graph.is_edge(1, 3));
        assert!(!graph.is_edge(2, 2));
        assert!(!graph.is_edge(2, 3));
        assert!(!graph.is_edge(3, 0));
        assert!(!graph.is_edge(3, 1));
        assert!(!graph.is_edge(3, 2));
        assert!(!graph.is_edge(3, 3));
    }

    #[test]
    fn test_to_dot_directed() {
        let graph: directed::AdjacencySets = directed::AdjacencySets::from_edges_with_order(4, [(0, 1), (1, 2), (2, 0)]);
        let mut dot = String::new();
        graph.to_dot(&mut dot).unwrap();
        assert_eq!(dot, indoc::indoc! {r#"
            strict digraph  {
                3;
                0 -> 1;
                1 -> 2;
                2 -> 0;
            }
        "#});
    }

    #[test]
    fn test_to_dot_undirected() {
        let graph: undirected::AdjacencySets = undirected::AdjacencySets::from_edges_with_order(4, [(0, 1), (0, 2), (1, 2)]);
        let mut dot = String::new();
        graph.to_dot(&mut dot).unwrap();
        assert_eq!(dot, indoc::indoc! {r#"
            strict graph  {
                3;
                0 -- 1;
                0 -- 2;
                1 -- 2;
            }
        "#});
    }

    #[test]
    fn test_gnp_dot_roundtrip() {
        let graph: directed::AdjacencySets = directed::AdjacencySets::gnp(500, 0.01);
        let mut dot = String::new();
        graph.to_dot(&mut dot).unwrap();
        let parsed = directed_from_dot(dot.as_str()).unwrap();
        assert_eq!(graph.order(), parsed.order());
        assert_eq!(graph.size(), parsed.size());
        assert_eq!(graph, parsed);
    }
}