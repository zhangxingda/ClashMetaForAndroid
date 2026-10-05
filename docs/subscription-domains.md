# Subscription device metadata domains

Device UUID, manufacturer and model are attached only to HTTPS subscription requests for these exact hosts (including custom domain aliases):

- v-ip9.pages.dev
- vip-5.pages.dev
- vip1.959621.xyz
- vip2-x3w.pages.dev
- vip2.959621.xyz
- vip2027-1.pages.dev
- vip4-2jc.pages.dev
- vip4.959621.xyz
- vip5.959621.xyz
- vip7-aiz.pages.dev
- vip7.959621.xyz
- vip9.959621.xyz

New Cloudflare domain aliases require adding their exact host to SubscriptionDeviceInfo.subscriptionHosts and rebuilding the APK. Other destinations receive no device metadata. Existing query parameters are preserved.
