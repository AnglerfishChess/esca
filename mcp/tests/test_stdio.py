"""The server as a client meets it: a stdio handshake, then a tool listing."""

import json
import subprocess
import sys

import pytest

#: What the server must offer once it is up.
TOOLS = {
    "position",
    "legal_moves",
    "explain_move",
    "facts",
    "opening",
    "book_moves",
    "pgn",
    "to_pgn",
}

#: The protocol revision the handshake asks for.
PROTOCOL = "2025-06-18"


class Client:
    """A JSON-RPC conversation with the server over its own stdin and stdout."""

    def __init__(self, process: subprocess.Popen[str]) -> None:
        self.process = process
        self.next_id = 0

    def send(self, method: str, params: dict | None = None) -> dict:
        """Sends one request and reads the answer to it."""
        self.next_id += 1
        self.notify(method, params, request_id=self.next_id)
        assert self.process.stdout is not None
        while line := self.process.stdout.readline():
            message = json.loads(line)
            if message.get("id") == self.next_id:
                return message
        raise AssertionError(f"the server answered nothing to {method}")

    def notify(self, method: str, params: dict | None = None, request_id: int | None = None) -> None:
        """Writes one message, expecting no answer."""
        message: dict = {"jsonrpc": "2.0", "method": method}
        if request_id is not None:
            message["id"] = request_id
        if params is not None:
            message["params"] = params
        assert self.process.stdin is not None
        self.process.stdin.write(json.dumps(message) + "\n")
        self.process.stdin.flush()


@pytest.fixture
def client():
    """A server started as a subprocess, shut down when the test is done."""
    process = subprocess.Popen(
        [sys.executable, "-m", "chess_esca_mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
        bufsize=1,
    )
    try:
        yield Client(process)
    finally:
        process.terminate()
        process.wait(timeout=10)


def test_the_server_completes_a_handshake_and_lists_its_tools(client):
    initialised = client.send(
        "initialize",
        {
            "protocolVersion": PROTOCOL,
            "capabilities": {},
            "clientInfo": {"name": "chess-esca-mcp tests", "version": "0"},
        },
    )
    assert initialised["result"]["serverInfo"]["name"] == "chess-esca"
    client.notify("notifications/initialized")

    listed = client.send("tools/list")
    assert {tool["name"] for tool in listed["result"]["tools"]} == TOOLS
    for tool in listed["result"]["tools"]:
        assert tool["description"]
        assert tool["annotations"]["readOnlyHint"] is True


def test_a_tool_called_over_the_wire_answers_structured_content(client):
    client.send(
        "initialize",
        {"protocolVersion": PROTOCOL, "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}},
    )
    client.notify("notifications/initialized")

    called = client.send("tools/call", {"name": "position", "arguments": {"fen": None}})
    answer = json.loads(called["result"]["content"][0]["text"])
    assert answer["side_to_move"] == "white"
    assert answer["legal_move_count"] == 20


def test_the_facts_schema_is_served_as_a_resource(client):
    client.send(
        "initialize",
        {"protocolVersion": PROTOCOL, "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}},
    )
    client.notify("notifications/initialized")

    listed = client.send("resources/list")
    assert [resource["uri"] for resource in listed["result"]["resources"]] == ["esca://facts-schema"]

    read = client.send("resources/read", {"uri": "esca://facts-schema"})
    schema = json.loads(read["result"]["contents"][0]["text"])
    assert schema["schema_id"]
    assert {group["name"] for group in schema["groups"]} >= {"material", "tactics"}


def test_the_analysis_prompt_is_served(client):
    client.send(
        "initialize",
        {"protocolVersion": PROTOCOL, "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}},
    )
    client.notify("notifications/initialized")

    listed = client.send("prompts/list")
    assert [prompt["name"] for prompt in listed["result"]["prompts"]] == ["analyse-position"]
