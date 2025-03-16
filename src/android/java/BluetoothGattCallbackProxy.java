package com.github.alexmoon.bluest.android;

import android.bluetooth.BluetoothGatt;
import android.bluetooth.BluetoothGattCharacteristic;
import android.bluetooth.BluetoothGattDescriptor;
import android.bluetooth.BluetoothGattCallback;

public class BluetoothGattCallbackProxy extends BluetoothGattCallback {
    private long arc;

    public BluetoothGattCallbackProxy(long arc) {
        this.arc = arc;
    }

    @Override
    public void onCharacteristicChanged(BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, byte[] value) {
        super.onCharacteristicChanged(gatt, characteristic, value);
        nativeOnCharacteristicChanged(this.arc, gatt, characteristic, value);
    }
    
    private native void nativeOnCharacteristicChanged(long arc, BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, byte[] value);
    
    @Override
    public void onCharacteristicChanged(BluetoothGatt gatt, BluetoothGattCharacteristic characteristic) {
        super.onCharacteristicChanged(gatt, characteristic);
        nativeOnCharacteristicChanged(this.arc, gatt, characteristic);
    }

    private native void nativeOnCharacteristicChanged(long arc, BluetoothGatt gatt, BluetoothGattCharacteristic characteristic);

    @Override
    public void onCharacteristicRead(BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, byte[] value, int status) {
        super.onCharacteristicRead(gatt, characteristic, value, status);
        nativeOnCharacteristicRead(this.arc, gatt, characteristic, value, status);
    }

    private native void nativeOnCharacteristicRead(long arc, BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, byte[] value, int status);

    @Override
    public void onCharacteristicRead(BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, int status) {
        super.onCharacteristicRead(gatt, characteristic, status);
        nativeOnCharacteristicRead(this.arc, gatt, characteristic, status);
    }

    private native void nativeOnCharacteristicRead(long arc, BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, int status);

    @Override
    public void onCharacteristicWrite(BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, int status) {
        super.onCharacteristicWrite(gatt, characteristic, status);
        nativeOnCharacteristicWrite(this.arc, gatt, characteristic, status);
    }

    private native void nativeOnCharacteristicWrite(long arc, BluetoothGatt gatt, BluetoothGattCharacteristic characteristic, int status);

    @Override
    public void onConnectionStateChange(BluetoothGatt gatt, int status, int newState) {
        super.onConnectionStateChange(gatt, status, newState);
        nativeOnConnectionStateChange(this.arc, gatt, status, newState);
    }

    private native void nativeOnConnectionStateChange(long arc, BluetoothGatt gatt, int status, int newState);

    @Override
    public void onDescriptorRead(BluetoothGatt gatt, BluetoothGattDescriptor descriptor, int status, byte[] value) {
        super.onDescriptorRead(gatt, descriptor, status, value);
        nativeOnDescriptorRead(this.arc, gatt, descriptor, status, value);
    }

    private native void nativeOnDescriptorRead(long arc, BluetoothGatt gatt, BluetoothGattDescriptor descriptor, int status, byte[] value);

    @Override
    public void onDescriptorRead(BluetoothGatt gatt, BluetoothGattDescriptor descriptor, int status) {
        super.onDescriptorRead(gatt, descriptor, status);
        nativeOnDescriptorRead(this.arc, gatt, descriptor, status);
    }

    private native void nativeOnDescriptorRead(long arc, BluetoothGatt gatt, BluetoothGattDescriptor descriptor, int status);

    @Override
    public void onDescriptorWrite(BluetoothGatt gatt, BluetoothGattDescriptor descriptor, int status) {
        super.onDescriptorWrite(gatt, descriptor, status);
        nativeOnDescriptorWrite(this.arc, gatt, descriptor, status);
    }

    private native void nativeOnDescriptorWrite(long arc, BluetoothGatt gatt, BluetoothGattDescriptor descriptor, int status);

    @Override
    public void onMtuChanged(BluetoothGatt gatt, int mtu, int status) {
        super.onMtuChanged(gatt, mtu, status);
        nativeOnMtuChanged(this.arc, gatt, mtu, status);
    }

    private native void nativeOnMtuChanged(long arc, BluetoothGatt gatt, int mtu, int status);

    @Override
    public void onPhyRead(BluetoothGatt gatt, int txPhy, int rxPhy, int status) {
        super.onPhyRead(gatt, txPhy, rxPhy, status);
        nativeOnPhyRead(this.arc, gatt, txPhy, rxPhy, status);
    }

    private native void nativeOnPhyRead(long arc, BluetoothGatt gatt, int txPhy, int rxPhy, int status);

    @Override
    public void onPhyUpdate(BluetoothGatt gatt, int txPhy, int rxPhy, int status) {
        super.onPhyUpdate(gatt, txPhy, rxPhy, status);
        nativeOnPhyUpdate(this.arc, gatt, txPhy, rxPhy, status);
    }

    private native void nativeOnPhyUpdate(long arc, BluetoothGatt gatt, int txPhy, int rxPhy, int status);

    @Override
    public void onReadRemoteRssi(BluetoothGatt gatt, int rssi, int status) {
        super.onReadRemoteRssi(gatt, rssi, status);
        nativeOnReadRemoteRssi(this.arc, gatt, rssi, status);
    }

    private native void nativeOnReadRemoteRssi(long arc, BluetoothGatt gatt, int rssi, int status);

    @Override
    public void onReliableWriteCompleted(BluetoothGatt gatt, int status) {
        super.onReliableWriteCompleted(gatt, status);
        nativeOnReliableWriteCompleted(this.arc, gatt, status);
    }

    private native void nativeOnReliableWriteCompleted(long arc, BluetoothGatt gatt, int status);

    @Override
    public void onServiceChanged(BluetoothGatt gatt) {
        super.onServiceChanged(gatt);
        nativeOnServiceChanged(this.arc, gatt);
    }

    private native void nativeOnServiceChanged(long arc, BluetoothGatt gatt);

    @Override
    public void onServicesDiscovered(BluetoothGatt gatt, int status) {
        super.onServicesDiscovered(gatt, status);
        nativeOnServicesDiscovered(this.arc, gatt, status);
    }

    private native void nativeOnServicesDiscovered(long arc, BluetoothGatt gatt, int status);

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
