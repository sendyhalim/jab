# Cid
CID is a database state management tool, think of it as git but for database. You can commit your current db state and checkout to your previous db state.


## Installation
### Manual (requires rust)
```
git clone git@github.com:sendyhalim/cid.git

make install
```

### Download
TODO

### Usage
```
# First create project.
# Currently only supports postgres.
cid project create awesomestuff --database-uri=asd

# Start commiting your db
cid project commit awesomestuff --message "my first commit"

# See log
cid project log awesomestuff

# Restore your db
cid project restore awesomestuff <hash>

# List of available projects
cid project list
```
