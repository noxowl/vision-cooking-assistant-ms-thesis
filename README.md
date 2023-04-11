# VAs-got-vision

## Features
* Recognize Human voice (VAD).
* Execute marker detection process by Human voice.
* Detect ArUco Marker and find nearest from gaze (fallback is centroid of frame or 0,0).

## Current screenshots
![](resources/images/test_1.png)
![](resources/images/test_2.png)

## How to use
1. Need to .env for compile and execute (changeme).
```
PICOVOICE_ACCESS_KEY=keyhere
PICOVOICE_MIC_INDEX=indexhere
```

2. ~ init command for generate markers.
3. ~ run command for execute.

## TODO
* Order sentence recognition.
* Link to IoT device and Alexa API.

### 개인용 마일스톤 (Korean)
* 공통
  * [ ] IoT 기능 (전구)
  * [ ] IoT 기능 (적외선 해야 할 것 같은데...)
  * [ ] 이게 뭐야 쿼리 기능
  * [ ] 날씨 쿼리 기능 (알렉사로 넘기기?)
  * [X] Picovoice 음성인식
* 비전
  * [X] 비전용 메시지센터
  * [X] 마커인식
  * [ ] 오브젝트 인식
* 논비전
  * [ ] 인식 안된 명령 폴백
  * [ ] 논비전용 메시지센터
* 시나리오
  * [ ] 1인 시나리오
  * [ ] 2인 시나리오