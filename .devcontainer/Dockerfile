FROM mcr.microsoft.com/devcontainers/base:bookworm

ARG ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT} \
    ANDROID_HOME=${ANDROID_SDK_ROOT} \
    PATH=${PATH}:${ANDROID_SDK_ROOT}/platform-tools:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin:${ANDROID_SDK_ROOT}/emulator

# GUI + GL bits for emulator window + hardware accel
RUN apt-get update && apt-get install -y --no-install-recommends \
    unzip wget curl git ca-certificates \
    libglu1-mesa mesa-utils libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxcursor1 libxi6 libxtst6 libxrandr2 libxdamage1 \
    libasound2 udev \
    qemu-kvm libvirt-daemon-system libvirt-clients \
  && rm -rf /var/lib/apt/lists/*

ENV DEBIAN_FRONTEND=noninteractive

# Install a JDK for sdkmanager (headless is fine)
RUN apt-get update && apt-get install -y --no-install-recommends \
    openjdk-17-jdk-headless \
    && rm -rf /var/lib/apt/lists/*

# Point JAVA_HOME and put it on PATH
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
ENV PATH=${JAVA_HOME}/bin:${PATH}

# Android cmdline tools + SDK/AVD components (API 34 example)
RUN mkdir -p ${ANDROID_SDK_ROOT}/cmdline-tools \
  && cd /tmp \
  && wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip -O cmdtools.zip \
  && unzip -q cmdtools.zip -d ${ANDROID_SDK_ROOT}/cmdline-tools \
  && mv ${ANDROID_SDK_ROOT}/cmdline-tools/cmdline-tools ${ANDROID_SDK_ROOT}/cmdline-tools/latest \
  && yes | sdkmanager --licenses || true \
  && sdkmanager "platform-tools" "platforms;android-34" "build-tools;34.0.0" "emulator" "system-images;android-34;google_apis;x86_64"

# Optional: pre-create an AVD definition to speed up first boot (still customizable via script)
# (We keep creation in a script so devs can re-create cleanly.)

# Corepack is enabled by postCreateCommand in devcontainer.json
