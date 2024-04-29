# coretime-renewal-bot

This is a bot to keep track of when a core becomes renewable, and when it does, renew it using the balance of `alice`.

For this, it checks the Coretime Chain's blocks for the desired core's `Renewable`
event, which indicates the core became renewable, then checks that the balance 
of the account has enough funds for the renewal and calls `broker.renew(core)`.
