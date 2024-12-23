# ALAS (Automatic Live Audio Streamer)

## Support / Donationware

Commercial support is available for purchase from Ridgeline Radio to support our non-commercial, non-profit radio station.

Stations with budgets over $100,000/year who use ALAS are encouraged to support the project in one of a few ways:

1. Buying the equipment from us
2. Paying monthly subscription fee for the bonded VPN
3. Buying a support package

## About ALAS

ALAS is a hardware/software appliance package that automatically streams and records audio with high reliability.
Whenever audio is present on the sound card input, ALAS will begin streaming and recording. If streaming should stop,
recording will continue, and vice versa. It achieves high straming reliability through the usage of a bonding VPN, with the
designs calling for both a cellular modem and usage of built-in WiFi. ALAS will discontinue streaming and recording after
a configurable period of silence.

## Project Navigation

### alas

The main application, written in Rust, which is deployed onto a Raspberry Pi. It is responsible for:

* Allowing for easy field configuration of the appliance via a WiFi access point
* Managing the WiFi state once configured by the end user
* Manging the cellular state once configured by the end user
* Continuously monitoring input audio signal to determine when audio is being input
* Stream audio to Icecast when audio is present
* Providing a web interface for end users to administer the device
* Ensuring audio traffic is using the appropriate Engarde Wireguard interface

### cad

You'll find FreeCAD and STEP files in this folder for fabricating the rack mountable case.

### docs

You'll find instructions on how to assemble the hardware and user's guide in this folder. Documentation for software
is generally found adjacent to these components.

### frontend

To save bandwidth, a static frontend is available with a configurable backend. The static frontend is found here.
This is distinct from the "deployed frontend," which is used during the initial configuration of the network settings
on the Raspberry Pi.

### infrastructure

This is the Terraform used to create the bonding VPN servers in Microsoft Azure. 

## ALAS Software Components

### alas_lib

alas_lib is meant to serve as the "brains" of the operation. However, there is some serious
separation of concerns issues here, as the main state of the application is actually held in `alas`.

### alas

This is the monolithic application that runs on the Raspberry Pi.