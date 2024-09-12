<!-- PROJECT SHIELDS -->

[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![MIT License][license-shield]][license-url]

<!-- PROJECT LOGO -->
<br />
<div align="center">
  <!-- <a href="https://github.com/mgnfy-view/soul-streams">
    <img src="assets/icon.svg" alt="Logo" width="80" height="80">
  </a> -->

  <h3 align="center">Soul Streams</h3>

  <p align="center">
    Soul streams is a payment streaming service on the Solana blockchain
    <br />
    <a href="https://github.com/mgnfy-view/soul-streams/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    ·
    <a href="https://github.com/mgnfy-view/soul-streams/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>

<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <li><a href="#contact">Contact</a></li>
  </ol>
</details>

<!-- ABOUT THE PROJECT -->

## About The Project

Soul streams is a payment streaming service on Solana. It allows anyone to create payment streams directed to any wallet and fund it with SPL tokens. The tokens are unlocked for the payee linearly over time. Additionally, the payer can cancel the stream which transfers the remaining balance of the stream back to them. The payer can also replenish their stream to start streaming tokens to the same payee again.

### Built With

-   ![Anchor](https://img.shields.io/badge/-ANCHOR-%23007ACC.svg?style=for-the-badge)
-   ![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
-   ![TypeScript](https://img.shields.io/badge/typescript-%23007ACC.svg?style=for-the-badge&logo=typescript&logoColor=white)
-   ![Yarn](https://img.shields.io/badge/yarn-%232C8EBB.svg?style=for-the-badge&logo=yarn&logoColor=white)

<!-- GETTING STARTED -->

## Getting Started

### Prerequisites

Make sure you have rust, solana-cli, anchor, git, node.js, and yarn installed and configured on your system.

### Installation

Clone the repo,

```shell
git clone https://github.com/mgnfy-view/soul-streams.git
```

cd into the repo, and install the necessary dependencies

```shell
cd soul-streams
yarn install
anchor build
```

Run any of the test files in the `./tests` folder by executing

```shell
anchor test ./tests/<test-file-name>.ts
```

That's it, you are good to go now!

<!-- ROADMAP -->

## Roadmap

-   [x] Solana program development
-   [x] Unit tests
-   [x] Write a good README.md

See the [open issues](https://github.com/mgnfy-view/soul-streams/issues) for a full list of proposed features (and known issues).

<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<!-- LICENSE -->

## License

Distributed under the MIT License. See `LICENSE.txt` for more information.

<!-- CONTACT -->

## Reach Out

Here's a gateway to all my socials, don't forget to hit me up!

[![Linktree](https://img.shields.io/badge/linktree-1de9b6?style=for-the-badge&logo=linktree&logoColor=white)][linktree-url]

<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/mgnfy-view/soul-streams.svg?style=for-the-badge
[contributors-url]: https://github.com/mgnfy-view/soul-streams/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/mgnfy-view/soul-streams.svg?style=for-the-badge
[forks-url]: https://github.com/mgnfy-view/soul-streams/network/members
[stars-shield]: https://img.shields.io/github/stars/mgnfy-view/soul-streams.svg?style=for-the-badge
[stars-url]: https://github.com/mgnfy-view/soul-streams/stargazers
[issues-shield]: https://img.shields.io/github/issues/mgnfy-view/soul-streams.svg?style=for-the-badge
[issues-url]: https://github.com/mgnfy-view/soul-streams/issues
[license-shield]: https://img.shields.io/github/license/mgnfy-view/soul-streams.svg?style=for-the-badge
[license-url]: https://github.com/mgnfy-view/soul-streams/blob/master/LICENSE.txt
[linktree-url]: https://linktr.ee/mgnfy.view
