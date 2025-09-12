import subprocess
from fastmcp import FastMCP

mcp = FastMCP(
    name="Semtools MCP Server"
)

@mcp.tool
def execute_bash(command: str) -> str:
    """Useful for executing bash commands on your machine."""
    return subprocess.run(command, shell=True, capture_output=True, text=True, timeout=60).stdout


if __name__ == "__main__":
    mcp.run()