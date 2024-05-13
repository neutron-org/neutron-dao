### Neutron Chain Manager

This contract implements a **chain management** **model** with two types of permission strategies:

1. **ALLOW_ALL**: gives a given address full access to the admin module, allowing to submit all possible types of privileged messages;
2. **ALLOW_ONLY**: allows a given address to submit privileged messages of a specific type, with further restrictions if applicable (see below).
