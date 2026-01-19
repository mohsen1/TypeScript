use ts_scanner::*;

fn main() {
    let source = r#"
// TypeScript example file
import { Component } from 'react';

interface Props {
    name: string;
    count?: number;
}

interface State {
    items: string[];
}

class MyComponent extends Component<Props, State> {
    private readonly id: number;

    constructor(props: Props) {
        super(props);
        this.id = Math.random();
        this.state = { items: [] };
    }

    async fetchItems(): Promise<void> {
        const response = await fetch('/api/items');
        const data = await response.json();
        this.setState({ items: data ?? [] });
    }

    render() {
        const { name, count = 0 } = this.props;
        const { items } = this.state;

        return (
            <div className={`container-${this.id}`}>
                <h1>{name}</h1>
                <span>{count}</span>
                {items.map((item, i) => (
                    <p key={i}>{item}</p>
                ))}
            </div>
        );
    }
}

export default MyComponent;
"#;

    let result = scan_all_tokens(source);

    println!("Total tokens: {}", result.tokens.len());
    println!("Has errors: {}", result.has_errors);

    // Print last 30 tokens
    for (i, token) in result.tokens.iter().skip(133).enumerate() {
        println!("{}: {:?}: '{}'", i + 133, token.kind, token.text(source));
    }

    // Check for export
    let has_export = result.tokens.iter().any(|t| t.kind == TokenKind::Export);
    println!("\nHas export: {}", has_export);
}
