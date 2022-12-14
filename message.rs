
use neutron_bindings::msg::{NeutronMsg, };



fn main() {
    title = "title";
    description = "description";
    let proposal = TextProposal { title, description };

    let msg = NeutronMsg::submit_param_change_proposal(proposal);
    println!(msg)
}
