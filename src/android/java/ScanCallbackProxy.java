package com.github.alexmoon.bluest.android;

import android.bluetooth.le.ScanCallback;
import android.bluetooth.le.ScanResult;

public class ScanCallbackProxy extends ScanCallback {
    private long arc;

    public ScanCallbackProxy(long arc) {
        this.arc = arc;
    }

    @Override
    public void onScanResult(int callbackType, ScanResult result) {
        super.onScanResult(callbackType, result);
        nativeOnScanResult(this.arc, callbackType, result);
    }

    private native void nativeOnScanResult(long arc, int callbackType, ScanResult result);

    @Override
    public void onScanFailed(int errorCode) {
        super.onScanFailed(errorCode);
        nativeOnScanFailed(this.arc, errorCode);
    }

    private native void nativeOnScanFailed(long arc, int errorCode);

    @Override
    protected void finalize() throws Throwable {
        try {
            nativeFinalize(this.arc);
        } finally {
            super.finalize();
        }
    }

    private native void nativeFinalize(long arc);
}
