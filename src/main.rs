mod model;
mod parser;
mod type_parser;
mod module_parser;
mod cache;
mod commands;
mod mcp;

use clap::Parser;

#[derive(Parser)]
#[command(name = "v8md", about = "1C metadata configuration CLI tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[command(about = "Load/update XML dump into cache")]
    LoadDump {
        #[arg(help = "Path to the directory containing Configuration.xml")]
        path: String,
    },
    #[command(about = "Get metadata for a 1C object")]
    Get {
        #[arg(help = "Object name, e.g. Document.Заказ, Catalog.Контрагенты, InformationRegister.КурсыВалют, Enum.СостоянияЗаказов, Constant.ВалютаУчета")]
        object: String,
        #[arg(short, long, help = "Show only attributes (properties)")]
        r#attributes: bool,
        #[arg(short, long, help = "Show only tabular sections")]
        tabular: bool,
    },
    #[command(about = "Get types of a specific attribute")]
    Types {
        #[arg(help = "Path to attribute, e.g. Document.Заказ.Покупатель")]
        path: String,
    },
    #[command(about = "List all cached objects")]
    List {
        #[arg(help = "Filter by kind: Document, Catalog, InformationRegister, AccumulationRegister, ChartOfCharacteristicTypes, Enum, Constant, DocumentJournal, ExchangePlan")]
        kind: Option<String>,
    },
    #[command(about = "Run as MCP server (JSON-RPC over stdio)")]
    Serve,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::LoadDump { path } => {
            let p = std::path::PathBuf::from(&path);
            match commands::load_dump(&p) {
                Ok(msg) => println!("{}", msg),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Get {
            object,
            r#attributes,
            tabular,
        } => match commands::get_object(&object, r#attributes, tabular) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Types { path } => match commands::get_types(&path) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::List { kind } => match commands::list_objects(kind.as_deref()) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Serve => {
            mcp::serve();
        }
    }
}