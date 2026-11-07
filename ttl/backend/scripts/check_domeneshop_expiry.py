#!/usr/bin/env python3
"""Notify the TTL admin Discord channel when the Domeneshop token is expiring."""

from __future__ import annotations

import argparse
import json
import os
import sys
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any
from urllib import error, request

DEFAULT_CREDENTIALS = Path.home() / "domeneshop_api.json"
DEFAULT_CONFIGURATION = Path("/srv/ttl-api/configuration.json")
DATE_FORMAT = "%Y-%m-%d %H:%M:%S"


def load_json(path: Path) -> dict[str, Any]:
    try:
        with path.open(encoding="utf-8") as file:
            data = json.load(file)
    except (OSError, json.JSONDecodeError) as error:
        raise RuntimeError(f"Unable to read JSON file {path}: {error}") from error

    if not isinstance(data, dict):
        raise RuntimeError(f"Expected a JSON object in {path}")
    return data


def parse_expiry(value: Any) -> datetime:
    if not isinstance(value, str):
        raise RuntimeError("domeneshop_expires must be a timestamp string")
    try:
        return datetime.strptime(value, DATE_FORMAT)
    except ValueError as error:
        raise RuntimeError(
            f"domeneshop_expires must use {DATE_FORMAT!r}, got {value!r}"
        ) from error


def build_message(expires_at: datetime, now: datetime, threshold_days: int) -> str:
    remaining = expires_at - now
    remaining_days = remaining.total_seconds() / 86400
    if remaining.total_seconds() < 0:
        status = "has expired"
    else:
        status = f"expires in {remaining_days:.1f} days"
    return (
        "Domeneshop API credentials need attention: "
        f"{status} ({expires_at.strftime(DATE_FORMAT)}). "
        f"Alert threshold: {threshold_days} days."
    )


def should_notify(expires_at: datetime, now: datetime, threshold_days: int) -> bool:
    return expires_at < now + timedelta(days=threshold_days)


def send_discord(webhook_url: str, message: str) -> None:
    if not webhook_url.startswith("https://"):
        raise RuntimeError("discord_admin_webhook_url must use HTTPS")

    payload = json.dumps({"content": message}).encode("utf-8")
    request_data = request.Request(
        webhook_url,
        data=payload,
        headers={
            "Accept": "application/json",
            "Content-Type": "application/json",
            "User-Agent": "TTL-Domeneshop-Expiry/1.0",
        },
        method="POST",
    )
    try:
        with request.urlopen(request_data, timeout=15) as response:
            if response.status < 200 or response.status >= 300:
                raise RuntimeError(f"Discord returned HTTP {response.status}")
    except error.HTTPError as error:
        try:
            response_body = error.read().decode("utf-8", errors="replace")
        except OSError:
            response_body = "<response body unavailable>"
        raise RuntimeError(
            f"Discord rejected the webhook with HTTP {error.code}: {response_body[:500]}"
        ) from error
    except OSError as error:
        raise RuntimeError(f"Unable to send Discord notification: {error}") from error


def run(
    credentials_path: Path,
    configuration_path: Path,
    now: datetime,
    threshold_days: int = 30,
) -> bool:
    if threshold_days <= 0:
        raise RuntimeError("threshold_days must be greater than zero")

    credentials = load_json(credentials_path)
    expires_at = parse_expiry(credentials.get("domeneshop_expires"))
    if not should_notify(expires_at, now, threshold_days):
        return False

    configuration = load_json(configuration_path)
    webhook_url = configuration.get("discord_admin_webhook_url")
    if not isinstance(webhook_url, str) or not webhook_url:
        raise RuntimeError("discord_admin_webhook_url is missing from configuration")

    send_discord(webhook_url, build_message(expires_at, now, threshold_days))
    return True


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--credentials",
        type=Path,
        default=Path(os.environ.get("DOMENESHOP_CREDENTIALS", DEFAULT_CREDENTIALS)),
        help="Path to the Domeneshop JSON file",
    )
    parser.add_argument(
        "--configuration",
        type=Path,
        default=Path(os.environ.get("TTL_CONFIGURATION", DEFAULT_CONFIGURATION)),
        help="Path to the TTL backend configuration JSON file",
    )
    parser.add_argument(
        "--threshold-days",
        type=int,
        default=30,
        help="Notify when expiry is sooner than this many days",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    now = datetime.now().astimezone().replace(tzinfo=None)
    try:
        notified = run(
            args.credentials,
            args.configuration,
            now,
            args.threshold_days,
        )
    except RuntimeError as error:
        print(f"domeneshop expiry check failed: {error}", file=sys.stderr)
        return 1

    if notified:
        print("Domeneshop expiry notification sent")
    else:
        print("Domeneshop credentials are outside the notification window")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
