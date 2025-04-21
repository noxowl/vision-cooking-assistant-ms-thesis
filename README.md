# Vision-Based Cooking Assistant (MS Thesis Prototype)

*MS Thesis Archive (2024)*

This repository contains the prototype implementation developed for my 2024 master's thesis as part of a **joint-degree program between Japan Advanced Institute of Science and Technology (JAIST) and Kanazawa University** (The degree was conferred by JAIST as the home institution).:

**"Voice Assistance Using Visual Information in Continuous Cooking Tasks to Extend User Experience"** Japan Advanced Institute of Science and Technology (JAIST), March 2024
> Internal project codename: `VAs-got-vision`

The system explores how voice-based cooking assistants can be enhanced by incorporating real-time visual context, such as object detection and environmental cues, to support continuous interaction and reduce user confusion during tasks.

To model the asynchronous nature of user behavior, perception, and system feedback in a cooking scenario, the prototype was implemented using an **actor-based system architecture in Rust**, where components such as visual processing, voice input, and interaction logic were isolated and coordinated via message passing.  

This design aimed to simulate real-time modular responsiveness in complex interaction flows.

> This code is not part of any published paper.  
> It is provided for archival purposes and technical reference only.

## Project Summary

- **Thesis Title:** _Voice Assistance Using Visual Information in Continuous Cooking Tasks to Extend User Experience_  
- **Institution:** Japan Advanced Institute of Science and Technology (JAIST)
- **Supervisor:** Prof. Takaya YUIZONO, [JAIST](https://www.jaist.ac.jp/laboratory/csd/yuizono.html) ([ORCID](https://orcid.org/0000-0002-9576-362X))
- **Second Supervisor:** Prof. Naoki SUGANUMA, [Kanazawa University](https://ridb.kanazawa-u.ac.jp/public/detail_en.php?id=2554&page=1&org1_cd=740000) ([J-Global](https://jglobal.jst.go.jp/en/detail/?JGLOBAL_ID=200901014342964957&t=1))
- **Completion:** March 2024
- **Status:** Research discontinued; not integrated into later projects.

### Technologies Used

- **Rust (Actor-based architecture)** for modular and asynchronous component communication
- OpenCV for visual input and experimental recognition pipelines
- Voice recognition: Picovoice SDK

## Disclaimer

- This code was part of an early-stage prototype and is **not production-ready**.
- Some parts may depend on deprecated APIs or internal tools.
- No official support is provided.

## Citation (IEEE style)

If you use or refer to this project, please cite it as follows:

[1] S. Rhie, “Voice Assistance Using Visual Information in Continuous Cooking Tasks to Extend User Experience”, M.S. thesis, Japan Advanced Institute of Science and Technology (JAIST), joint-degree program with Kanazawa University, 2024. [Online]. Available: https://github.com/noxowl/vision-cooking-assistant-ms-thesis

## License

This project is licensed under the [BSD 3-Clause License](LICENSE).

You are free to use, modify, and distribute this code for non-commercial or commercial purposes, provided that the original copyright notice and this license are included  
in all copies or substantial portions of the Software.

> Note: Redistribution using the names of the original author or affiliated institutions for endorsement is not permitted.
