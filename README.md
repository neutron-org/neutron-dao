# neutron-dao

This is very first version of neutron dao contract.  It has the simplest API, which implements 2 methods: 
- query vote power of specific user
- query all vote powers of all users

Both of them currently return hardcoded values.


# Testing 

1. from `neutron` run: `make init`
2. run `./test_proposal.sh`
3. see that proposal has passed
