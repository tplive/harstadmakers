import json
import tempfile
import unittest
from datetime import datetime
from pathlib import Path
from unittest.mock import patch

import check_domeneshop_expiry


class DomeneshopExpiryTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.root = Path(self.directory.name)
        self.credentials = self.root / "domeneshop_api.json"
        self.configuration = self.root / "configuration.json"
        self.configuration.write_text(
            json.dumps({"discord_admin_webhook_url": "https://discord.test/webhook"}),
            encoding="utf-8",
        )

    def tearDown(self):
        self.directory.cleanup()

    def write_credentials(self, expires: str):
        self.credentials.write_text(
            json.dumps({"domeneshop_expires": expires}),
            encoding="utf-8",
        )

    @patch("check_domeneshop_expiry.send_discord")
    def test_notifies_before_threshold(self, send_discord):
        self.write_credentials("2026-12-06 14:59:59")
        now = datetime(2026, 11, 6, 15, 0, 0)

        notified = check_domeneshop_expiry.run(
            self.credentials, self.configuration, now
        )

        self.assertTrue(notified)
        send_discord.assert_called_once()

    @patch("check_domeneshop_expiry.send_discord")
    def test_does_not_notify_at_or_after_threshold(self, send_discord):
        self.write_credentials("2026-12-06 15:00:00")
        now = datetime(2026, 11, 6, 15, 0, 0)

        notified = check_domeneshop_expiry.run(
            self.credentials, self.configuration, now
        )

        self.assertFalse(notified)
        send_discord.assert_not_called()

    @patch("check_domeneshop_expiry.send_discord")
    def test_notifies_for_expired_credentials(self, send_discord):
        self.write_credentials("2026-11-05 15:00:00")
        now = datetime(2026, 11, 6, 15, 0, 0)

        self.assertTrue(
            check_domeneshop_expiry.run(
                self.credentials, self.configuration, now
            )
        )
        send_discord.assert_called_once()


if __name__ == "__main__":
    unittest.main()
