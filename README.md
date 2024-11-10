## Preconfirmation Router

This is an open source preconfirmation router, this sits between the wallet and the various preconfirmation networks, it allows the wallet to request an inclusion or execution preconfirmation without having to think of the implementation details of the various preconfirmation networks.

This repo is under active development & changing frequently.

This currently connects to the following preconfirmation networks (a-z):
 - Bolt by Chainbound
 - EthGas (in progress)
 - Interstate
 - Luban (in progress)
 - Primev

If you would like to be included, please add a pull request!

![Preconfirmation Router Architecture](static/image.png)

Pricing
- Pricing is not yet implemented
- Ideally this router & all preconfirmation providers implement this spec eventually: https://www.notion.so/Pricing-Spec-for-Preconfirmations-13777b17f2e68064adacc0e4ee8a5353

You can test using the cli here: https://github.com/interstate-labs/preconf-cli

