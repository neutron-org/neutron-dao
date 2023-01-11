# Overrule proposal module

This is a pre-propose module that allows to create overrule type proposals.

It requires for the DAO to have separate proposal module for overrule proposals
(it should have less voting period as well as low voting power threshold).

This pre-proposal module
1. Restricts the creation of something other than overrule proposals
2. Provides the interface for simple overrule proposal creation

Essentially, this pre-proposal module just a wrapper for a proper proposal message.

Warning: no deposits allowed since deposits make no sense in context of overrule proposals.