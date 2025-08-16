FROM mcr.microsoft.com/devcontainers/base:bookworm

ARG ANDROID_SDK_ROOT=/opt/android-sdk
ENV ANDROID_SDK_ROOT=${ANDROID_SDK_ROOT} \
  ANDROID_HOME=${ANDROID_SDK_ROOT} \
  PATH=${PATH}:${ANDROID_SDK_ROOT}/platform-tools:${ANDROID_SDK_ROOT}/cmdline-tools/latest/bin:${ANDROID_SDK_ROOT}/emulator

# Base tools and libraries for emulator
RUN apt-get update && apt-get install -y --no-install-recommends \
  unzip wget curl git ca-certificates \
  libglu1-mesa mesa-utils libx11-6 libx11-xcb1 libxcb1 libxcomposite1 libxcursor1 libxi6 libxtst6 libxrandr2 libxdamage1 \
  libasound2 udev \
  && rm -rf /var/lib/apt/lists/*

# Tauri build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
  pkg-config \
  libgtk-3-dev \
  libsoup-3.0-dev \
  javascriptcoregtk-4.1 \
  libjavascriptcoregtk-4.1-dev \
  webkit2gtk-4.1 \
  && rm -rf /var/lib/apt/lists/*

# VNC and display server dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
  libxkbfile1 libpulse0 libxcb-icccm4 libxcb-image0 libxcb-keysyms1 \
  libxcb-randr0 libxcb-render-util0 libxcb-xinerama0 libxcb-xinput0 \
  libxcb-xfixes0 libxcb-util1 \
  xvfb x11vnc fluxbox \
  supervisor \
  net-tools \
  python3-websockify websockify \
  novnc \
  && rm -rf /var/lib/apt/lists/*

ENV DEBIAN_FRONTEND=noninteractive

# Install a JDK for sdkmanager (headless is fine)
RUN apt-get update && apt-get install -y --no-install-recommends \
  openjdk-17-jdk-headless \
  && rm -rf /var/lib/apt/lists/*

# Point JAVA_HOME and put it on PATH
ENV JAVA_HOME=/usr/lib/jvm/java-17-openjdk-amd64
ENV PATH=${JAVA_HOME}/bin:${PATH}

# Android cmdline tools + SDK/AVD components
RUN mkdir -p ${ANDROID_SDK_ROOT}/cmdline-tools \
  && cd /tmp \
  && wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip -O cmdtools.zip \
  && unzip -q cmdtools.zip -d ${ANDROID_SDK_ROOT}/cmdline-tools \
  && mv ${ANDROID_SDK_ROOT}/cmdline-tools/cmdline-tools ${ANDROID_SDK_ROOT}/cmdline-tools/latest \
  && yes | sdkmanager --licenses || true

# Install SDK components with x86_64 image for better performance in container
RUN sdkmanager "platform-tools" \
  "platforms;android-34" \
  "build-tools;34.0.0" \
  "emulator" \
  "system-images;android-34;google_apis;x86_64" \
  "ndk;25.1.8937393"

# Set up NDK environment
ENV ANDROID_NDK_HOME=${ANDROID_SDK_ROOT}/ndk/25.1.8937393
ENV NDK_HOME=${ANDROID_NDK_HOME}

# Create vscode user with proper permissions
RUN useradd -ms /bin/bash vscode && \
    usermod -aG sudo vscode && \
    echo "vscode ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# VNC configuration - use display :99 consistently
ENV DISPLAY=:99
ENV VNC_PORT=5901
ENV NOVNC_PORT=6080
ENV VNC_PASSWORD=android
ENV VNC_RESOLUTION=1280x1024
ENV VNC_DEPTH=24

# Create VNC directory (password will be set at runtime)
RUN mkdir -p /home/vscode/.vnc && \
    chown -R vscode:vscode /home/vscode/.vnc

# Create log directories
RUN mkdir -p /var/log/supervisor /var/log/vnc && \
    chmod 777 /var/log/vnc

# Create startup script that calls our VNC service manager
RUN echo '#!/bin/bash\n\
set -e\n\
if [ -f /workspaces/tauri-plugin-ankidroid/scripts/vnc/start-services.sh ]; then\n\
    /workspaces/tauri-plugin-ankidroid/scripts/vnc/start-services.sh\n\
else\n\
    echo "VNC start script not found"\n\
    exit 1\n\
fi\n\
' > /usr/local/bin/start-vnc.sh && \
chmod +x /usr/local/bin/start-vnc.sh

USER vscode
WORKDIR /home/vscode