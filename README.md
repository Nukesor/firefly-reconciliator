# Firefly Reconciliator

This tool is designed to provide a convenient and reproducable way to create
reconciliation entries for your firefly accounts via history replay.

Firefly's builtin reconciliation mechanism lacks a few features and is also inherently flawed.

1. **Static balances** \
  It's not possible to tell Firefly that an account had a certain balance at a certain point in time.
  This is useful if you got your statement of account and know exactly how much money was on that account at the end of the month. \
  Firefly's builtin reconciliations aren't automatically updated when a transaction is added or updated after a reconciliation has been created.
  For example:
    - A firefly account has a balance of `520€` at the end of October.
    - The actual balance on the respective real world bank account is `500€`.
    - We now manually create a reconciliation of `-20€` for that month.
    - You now find that missing receipt over `-20€` and add it with the correct date.
    - The reconciliation is not updated to `0€` and stays on `-20€` resulting in an firefly account balance of `480€`.
2. **Positive Reconciliations** \
  Firefly's reconciliations are negative by design. It's impossible to create a positive reconciliation.
  So if you ever got some money and forgot to add it to your account history, you won't be able to depict this with reconciliations.

To work around these issues, this little helper script takes a history of end-of-month bank account balance,
replays the whole transaction firefly transaction history and creates/updates "reconciliation transactions"
at the end of each month with the newest data. Every time the script is run.

However, these reconciliaton transactions aren't firefly's reconciliations, but they're rather normal `Deposit`s and `Withdrawal`s, due to the limitations mentioned above.

## Installation

1. Install library `cargo install --locked --path .`
1. Create a configuration file.
  Default location is at `~/.config/firefly_reconciliator.yml`
1. Add the bank data, firefly `token` and `base_url`
1. Run the script via `cargo run`.

## Configuration file

The configuration file consists of some operational config and your historic bank data.
The bank data is ordered by accounts.

```yml
token: "firefly_token"
base_url: http://localhost:8081
accounts:
  - name: YourBankName
    # The id of the account in firefly
    firefly_id: 2
    data:
      # The bank data for 2023
      2023:
          # Account balance at the end of each month.
          # I.e. `1: 51230` means that at the end of January, there
          # were 512.30 € on that account.
          1: 51230
          2: 512304
          3: 123456
          4: 789001
          5: 151235
          6: 161234
          7: 123152
          8: 151231
          9: 152315
      2022:
          1: 51230
          2: 512304
          3: 123456
          4: 789001
          5: 151235
          6: 161234
          7: 123152
          8: 151231
          9: 152315
          10: 151245
          11: 608029
          12: 607093
```
