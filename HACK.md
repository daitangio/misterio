
Some nice features you may like to explore:

1. You can hack the checkout branch (the default is the current branch, but you can use it another).
2. Do not forget to customize your .ssh/config file, to simplify misterio launch
3. You can set misterio public repo as upstream, merge to your code and kept your secret hosts/ dir in your private repository (hosts dir is ignored by design in git: you must force its versioning).
4. scp is known to be a dirty trick. Misterio try to use rsync and will fallback to scp only if not found.

Misterio ssh optimization
A bit more complex:
Copying the entire repo every time is stupid: one can think to sync from master repo.
One way is to fire a git server on the master server (the one running misterio-ssh), tunnel it via bash and let the client pull the changes.
You need git as extra dep on the client and it is a bit risky because you are opening a connection client --> master whereas now misterio is always master --> client (one way)