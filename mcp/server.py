import argparse
import subprocess
from typing import Dict, Any

from fastmcp import FastMCP

mcp = FastMCP(
    name="Semtools MCP Server"
)

@mcp.tool
def execute_bash(command: str) -> str:
    """Useful for executing bash commands on your machine.
    When executing bash commands, you have two very helpful utilities installed
    - `parse` -- converts any non grep-able format into markdown, outputs a filepath for a converted markdown file for every input file to stdin
    - `search` -- performs a search using static embeddings on either stdin or a list of files (very similar to grep). Works best with keyword based search queries. Only works with text-based files so it may require the `parse` tool to help preprocess into markdown.

    These command, combined with other CLI commands, you can ensure that you can search large amounts of files efficiently, while handling various formats of documents. Both `parse` and `search` can scale to hundreds of thousands of documents.

    ## Parse CLI Help

    ```bash
    parse --help
    A CLI tool for parsing documents using various backends

    Usage: parse [OPTIONS] <FILES>...

    Arguments:
    <FILES>...  Files to parse

    Options:
    -c, --parse-config <PARSE_CONFIG>  Path to the config file. Defaults to ~/.parse_config.json
    -b, --backend <BACKEND>            The backend type to use for parsing. Defaults to `llama-parse` [default: llama-parse]
    -h, --help                         Print help
    -V, --version                      Print version
    ```

    ## Search CLI Help

    ```bash
    search --help
    A CLI tool for fast semantic keyword search

    Usage: search [OPTIONS] <QUERY> [FILES]...

    Arguments:
    <QUERY>     Query to search for (positional argument)
    [FILES]...  Files or directories to search

    Options:
    -n, --n-lines <N_LINES>            How many lines before/after to return as context [default: 3]
        --top-k <TOP_K>                The top-k files or texts to return (ignored if max_distance is set) [default: 3]
    -m, --max-distance <MAX_DISTANCE>  Return all results with distance below this threshold (0.0+)
    -i, --ignore-case                  Perform case-insensitive search (default is false)
    -h, --help                         Print help
    -V, --version                      Print version
    ```
    """
    return subprocess.run(command, shell=True, capture_output=True, text=True, timeout=60).stdout

def _parse_cli_args() -> argparse.Namespace:
    """Build and parse CLI args for the MCP server entrypoint."""
    parser = argparse.ArgumentParser(description="Semtools MCP Server")
    parser.add_argument(
        "--transport",
        choices=("http", "stdio"),
        default="http",
        help="Transport protocol to use when serving the MCP interface.",
    )
    parser.add_argument(
        "--host",
        default="127.0.0.1",
        help="Host interface for the HTTP transport (default: 127.0.0.1).",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=9001,
        help="TCP port for the HTTP transport (default: 9001).",
    )
    return parser.parse_args()


if __name__ == "__main__":
    args = _parse_cli_args()
    run_kwargs: Dict[str, Any] = {"transport": args.transport}
    if args.transport == "http":
        run_kwargs.update(host=args.host, port=args.port)
    mcp.run(**run_kwargs)
