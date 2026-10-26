mod model;
mod parser;
mod type_parser;
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
        #[arg(long, help = "Project name (required). Cache of this project is rebuilt from scratch")]
        project: String,
    },
    #[command(about = "Get metadata for a 1C object")]
    Get {
        #[arg(help = "Object name, e.g. Document.Заказ, Catalog.Контрагенты, InformationRegister.КурсыВалют, Enum.СостоянияЗаказов, Constant.ВалютаУчета")]
        object: String,
        #[arg(short, long, help = "Show only attributes (properties)")]
        r#attributes: bool,
        #[arg(short, long, help = "Show only tabular sections")]
        tabular: bool,
        #[arg(long, help = "Project name (required)")]
        project: String,
    },
    #[command(about = "Get types of a specific attribute")]
    Types {
        #[arg(help = "Path to attribute, e.g. Document.Заказ.Покупатель")]
        path: String,
        #[arg(long, help = "Project name (required)")]
        project: String,
    },
    #[command(about = "List all cached objects of a project")]
    List {
        #[arg(help = "Filter by kind: Document, Catalog, InformationRegister, AccumulationRegister, ChartOfCharacteristicTypes, Enum, Constant, DocumentJournal, ExchangePlan")]
        kind: Option<String>,
        #[arg(long, help = "Project name (required)")]
        project: String,
    },
    #[command(about = "List loaded projects")]
    Projects,
    #[command(about = "Run as MCP server (JSON-RPC over stdio)")]
    Serve,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::LoadDump { path, project } => {
            let p = std::path::PathBuf::from(&path);
            match commands::load_dump(&p, &project) {
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
            project,
        } => match commands::get_object(&object, r#attributes, tabular, &project) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Types { path, project } => match commands::get_types(&path, &project) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::List { kind, project } => match commands::list_objects(kind.as_deref(), &project) {
            Ok(output) => println!("{}", output),
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Projects => match commands::projects_list() {
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