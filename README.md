# Firefly Reconciliator

This tool is designed to provide a convenient and reproducable way to create
reconciliation entries for your firefly accounts via history replay.

## Installation

1. Install library `cargo install --locked --path .`
1. Create a configuration file.
  Default location is at `~/.config/firefly_reconciliator.yml`
1.

## Configuration file

The configuration file consists of your firefly token and your historic bank data.
The bank data is ordered by accounts.

```yml
token: "firefly_token"
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
