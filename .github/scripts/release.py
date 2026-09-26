"""Resolve a release tag once and pass its immutable commit to every release job."""

import json
import os
import re
import subprocess
import sys
import tomllib
from pathlib import Path


def select_tag(event_name, event, workflow_ref, publish_requested):
    """Validate event policy and return a tag name, never a branch or raw revision."""
    if event_name == "release":
        tag = event["release"]["tag_name"]
        if workflow_ref != f"refs/tags/{tag}":
            raise ValueError("The release event must run from its own tag")
    elif event_name == "workflow_dispatch":
        tag = event["inputs"]["tag"]
        if publish_requested and workflow_ref != "refs/heads/main":
            raise ValueError("Manual publication must run the workflow from main")
    else:
        raise ValueError("Only release and workflow_dispatch events are supported")

    if not re.fullmatch(r"v[0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", tag):
        raise ValueError("Use a tag name such as v0.1.1, without a refs/ prefix")
    return tag


def validate_event_commit(event_name, event_sha, tag_sha):
    """Reject a release tag that has moved since the release event was created."""
    if event_name == "release" and event_sha != tag_sha:
        raise ValueError("The release tag no longer points to the event commit")


def release_identity(tag, sha, manifest):
    """Compute validated release metadata from the selected commit's manifest."""
    if not re.fullmatch(r"[0-9a-f]{40}", sha):
        raise ValueError("Expected a full commit SHA")
    package = manifest["package"]
    if package["name"] != "restqs":
        raise ValueError("The release must contain the restqs package")
    if tag != f"v{package['version']}":
        raise ValueError("The release tag must match the selected Cargo.toml version")
    return {"sha": sha, "tag": tag, "version": package["version"]}


def git_output(*arguments):
    """Read repository metadata without invoking a shell."""
    return subprocess.check_output(["git", *arguments], text=True).strip()


def require_main_ancestor(sha):
    """Require the tagged commit to have been merged into the default branch."""
    subprocess.run(
        ["git", "merge-base", "--is-ancestor", sha, "refs/remotes/origin/main"],
        check=True,
    )


def resolve_release(event_name, event, workflow_ref, event_sha, publish_requested):
    """Coordinate tag lookup, ancestry checks, and pure release validation."""
    tag = select_tag(event_name, event, workflow_ref, publish_requested)
    sha = git_output("rev-parse", "--verify", f"refs/tags/{tag}^{{commit}}")
    validate_event_commit(event_name, event_sha, sha)
    require_main_ancestor(sha)
    manifest = tomllib.loads(git_output("show", f"{sha}:Cargo.toml"))
    return release_identity(tag, sha, manifest)


def format_outputs(identity):
    """Format validated values for the GitHub Actions output file."""
    return "".join(f"{key}={value}\n" for key, value in identity.items())


def format_summary(identity):
    """Show the immutable revision that must pass checks before publication."""
    return (
        f"Release: `{identity['tag']}`\n\n"
        f"Commit: `{identity['sha']}`\n\n"
        "All checks and publication use this commit, even if the tag later moves.\n"
    )


def main():
    """Coordinate workflow input and output adapters."""
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text())
    identity = resolve_release(
        os.environ["GITHUB_EVENT_NAME"],
        event,
        os.environ["GITHUB_REF"],
        os.environ["GITHUB_SHA"],
        json.loads(os.environ["PUBLISH_REQUESTED"]),
    )
    with Path(os.environ["GITHUB_OUTPUT"]).open("a") as output:
        output.write(format_outputs(identity))
    with Path(os.environ["GITHUB_STEP_SUMMARY"]).open("a") as summary:
        summary.write(format_summary(identity))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, subprocess.CalledProcessError) as error:
        print(f"Release validation failed: {error}", file=sys.stderr)
        sys.exit(1)
