# Neutron dao

[![BSL 1.1 License][license-shield]][license-url]
<!--[![Website][neutron-shield]][neutron-url]-->
<!-- 
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
-->

<!-- PROJECT LOGO -->
<br />
<p align="center">
  <a href="https://github.com/neutron-org">
    <img src="https://avatars.githubusercontent.com/u/108675945?s=200&v=4" alt="Logo" width="80" height="80">
  </a>

<h2 align="center">Neutron - DAO</h2>

## Overview

The Neutron governance is based on [DAO DAO](https://github.com/DA0-DA0/dao-contracts) contracts, with some modifications. In addition, we implemented 3 contracts that manage Neutron’s funds.

- **The Neutron DAO**
- **Multiple subDAOs**
- **The Treasury** holds the vested NTRNs and sends them to the Reserve and Distribution contracts;
- **The Reserve** contract keeps the funds that have already vested, but were not sent to the Distribution contract. Is is used by the neutron DAO for one-off payouts;
- **The Distribution** contract is responsible of the second step of token distribution where tokens sent to this contract are distributed between `share holders`, where `share holders` are a configurable set of addresses with number of shares. This contract allows share holders to withdraw collected tokens.

The Neutron DAO creates subDAOs by executing Neutron DAO proposals that contain *Instantiate* messages for the subDAO contracts.

## Testing 

1. from `neutron` run: `make init`
2. run `./test_proposal.sh`
3. see that proposal has passed
4. as well everybody can run `./test_subdao_proposal.sh` with same results

We also have decent amount of integration tests 


## License

Distributed under the BSL 1.1 License. See `LICENSE` for more information.

<!-- CONTRIBUTING -->
## Contributing

Contributions are what makes the open source community such an amazing place to be learn, inspire, and create. Any contributions you make are **greatly appreciated**.

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request


<!-- CONTACT -->
## Contact

Neutron - [@Neutron_org](https://twitter.com/Neutron_org) - info (a) neutron.org

Project Link: [https://github.com/neutron-org/neutron-dao](https://github.com/neutron-org/neutron-dao)

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->


[license-shield]: https://img.shields.io/badge/license-BSL%201.1-green?style=for-the-badge
[license-url]: https://github.com/neutron-org/neutron-tests/blob/main/LICENSE.txt
[neutron-shield]: https://static.tildacdn.com/tild3833-3631-4236-b131-663933343237/3b1510ab-746d-4947-8.svg
[neutron-url]: https://neutron.org

[product-screenshot]: images/screenshot.png
