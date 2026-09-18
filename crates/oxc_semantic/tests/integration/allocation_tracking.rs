use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn scoping_allocation_stats(source: &str) -> (usize, usize) {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::mjs()).parse();
    assert!(parsed.diagnostics.is_empty());
    let parser_stats = allocator.get_allocation_stats();

    let result = SemanticBuilder::new().build(&parsed.program);
    assert!(result.diagnostics.is_empty());
    let scoping = result.semantic.scoping();
    assert_eq!(scoping.symbols_len(), 1);
    assert!(scoping.references_len() > 0);

    let stats = scoping.allocator_allocation_stats();
    // Reading the counters must not reset them or perform allocations.
    assert_eq!(scoping.allocator_allocation_stats(), stats);
    assert_eq!(allocator.get_allocation_stats(), parser_stats);
    stats
}

#[test]
fn tracks_scoping_allocations_not_parser_allocations() {
    let (allocs, _) = scoping_allocation_stats("let value = 0; value;");
    assert!(allocs > 0);
}

#[test]
fn tracks_scoping_reallocations() {
    // Grow a single symbol's reference list beyond its initial capacity without depending on
    // platform-specific HashMap growth or exact allocation counts.
    let source = format!("let value = 0; {}", "value;".repeat(64));
    let (allocs, reallocs) = scoping_allocation_stats(&source);
    assert!(allocs > 0);
    assert!(reallocs > 0);
}
