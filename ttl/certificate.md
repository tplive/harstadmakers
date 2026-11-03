# Issue certificate for Domeneshop

Using [Domeneshop](https://domene.shop) as the registrar.

Using [acme.sh](https://github.com/acmesh-official/acme.sh) as the automation tool. Install the tool with `curl https://get.acme.sh | sh -s email=my@example.com`

1. Log in to Domeneshop with your account.
2. Follow these instructions: [Use Domeneshop DNS API to automatically issue a cert](https://github.com/acmesh-official/acme.sh/wiki/dnsapi2#85-use-domeneshop-dns-api-to-automatically-issue-a-cert)
3. "You'll have to get a Domeneshop API key and secret (https://api.domeneshop.no/docs/)."
4. "To generate credentials, visit this page after logging in to the control panel on our website: https://www.domeneshop.no/admin?view=api"
5. Note the token, the secret and the expiry timestamp.
6. For manual issuance and automation, run:

```bash
export DOMENESHOP_Token=your_token_here
export DOMENESHOP_Secret=your_secret_here
acme.sh --issue --dns dns_domeneshop -d ttl.example.com -d ttl-api.example.com
```

7. To Keep the data local on the TTL server, in the admin user's `~`, put domeneshop_api.json:

```json
{
    "domeneshop_token": "************",
    "domeneshop_secret": "*********************************************************",
    "domeneshop_expires": "2027-11-03 21:54:38"
}
```

8. The expiry date has to be updated, as the Discord integration will use it to remind you to refresh the token 30 days before expiry. Mind you, the certificates will be renewed at a much higher cadence, at the moment 90 days.
