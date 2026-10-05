package com.github.kr328.clash.service

import android.content.Context
import android.net.Uri
import android.os.Build
import android.util.AtomicFile
import java.io.File
import java.util.UUID

/** App installation identity, kept outside Android's backup data. */
object SubscriptionDeviceInfo {
    private val parameterNames = setOf("device_id", "device_brand", "device_model")

    @Synchronized
    private fun installationId(context: Context): String {
        val file = AtomicFile(File(context.noBackupFilesDir, "subscription-device-id"))
        val saved = runCatching {
            UUID.fromString(String(file.readFully(), Charsets.UTF_8)).toString()
        }.getOrNull()
        if (saved != null) return saved

        val id = UUID.randomUUID().toString()
        val stream = file.startWrite()
        try {
            stream.write(id.toByteArray(Charsets.UTF_8))
            file.finishWrite(stream)
        } catch (error: Exception) {
            file.failWrite(stream)
            throw error
        }
        return id
    }

    fun forSubscription(context: Context, source: String): String {
        val uri = Uri.parse(source)
        // Only send device information to the user's subscription service.
        if (uri.scheme != "https" || uri.host != "vip2027-1.pages.dev") return source
        val builder = uri.buildUpon().clearQuery()
        for (name in uri.queryParameterNames) {
            if (name !in parameterNames) {
                for (value in uri.getQueryParameters(name)) builder.appendQueryParameter(name, value)
            }
        }
        return builder
            .appendQueryParameter("device_id", installationId(context))
            .appendQueryParameter("device_brand", Build.MANUFACTURER)
            .appendQueryParameter("device_model", Build.MODEL)
            .build().toString()
    }
}
