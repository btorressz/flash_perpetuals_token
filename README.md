# flash_perpetuals_token

## **Overview**
Flash Perpetuals Token ($FPT) is a **Solana-based perpetual program** designed for **high-frequency traders (HFT)** seeking **instant leveraged execution** with minimal latency. The protocol allows traders to **stake $FPT tokens**, access **leveraged perpetual positions**, and participate in a **liquidation and hedging system** that ensures market stability.

## **Key Features**
✅ **Ultra-Fast Execution:** Designed for HFT traders with **near-zero latency order execution**.  
✅ **Dynamic Liquidation System:** Supports **partial liquidations** to prevent complete position wipeouts.  
✅ **Merkle-Based Order Execution:** Uses **Merkle trees** to **prevent order spoofing** and enhance efficiency.  
✅ **Advanced Risk Management:** Implements **automated hedging**, **dynamic funding rate adjustments**, and **reputation-based trading limits**.  
✅ **Liquidity Provider (LP) Incentives:** LPs earn **real yield** from **funding rates and trading fees**.  
✅ **Flash Loan Resistance:** Implements **time-locked staking** to prevent flash loan exploits.  
✅ **Role-Based Access Control:** Restricts key operations to **admins, LPs, and whitelisted market makers**.  

---

## **Smart Contract(program) Architecture**
The **Flash Perpetuals Token** program consists of multiple **on-chain components** working together to facilitate **fast, secure perpetual trading**.

### **1️⃣ Global State (Protocol Configuration)**
The **GlobalState** account serves as the **central registry** for protocol-wide parameters:
- **Admin Authority:** The admin manages protocol settings, including **fee rates and liquidation parameters**.
- **Fee and Funding Rates:** Dynamically adjusts **funding payments between long and short positions**.
- **Merkle Orderbook Root:** A **Merkle tree-based order execution system** ensuring **tamper-proof, high-speed trading**.

### **2️⃣ Trader State (User Accounts)**
Each trader has a **TraderState account**, which stores:
- **Staked FPT Tokens:** Required to access **leveraged trading**.
- **Open Positions:** Tracks the trader’s **leverage and margin levels**.
- **Liquidation Risk:** If a trader’s **margin falls below maintenance**, they are **partially or fully liquidated**.

### **3️⃣ Liquidity Provider (LP) State**
LPs provide capital to support perpetual trading. The **LpState** account tracks:
- **Total LP Stake:** Liquidity providers must stake FPT to **earn trading fees**.
- **Trading Volume-Based Rewards:** LPs receive rewards **based on their contribution to liquidity depth**.
- **Hedging System Integration:** LP funds are automatically **hedged to minimize risk exposure**.

### **4️⃣ High-Frequency Trading Execution System**
To support **low-latency order execution**, the protocol includes:
- **Batch Trade Execution:** Traders can **submit multiple orders in a single transaction**.
- **Merkle-Based Order Validation:** Ensures **tamper-proof order verification**.
- **Time-Weighted Funding Rates:** Dynamically adjusts **funding payments based on volatility**.

---


## **Function Descriptions**
This section describes the key functions within the **Flash Perpetuals Token** program.

### **1️⃣ Initialization & Configuration**
#### **initialize**
- Initializes the **GlobalState** account.
- Sets the **admin authority, fee rate, margin levels, and execution fees**.

#### **add_authorized_liquidator**
- Grants **liquidation privileges** to a specified **liquidator or HFT firm**.

#### **add_authorized_orderbook_updater**
- Grants access to **orderbook updates** for **whitelisted market makers**.

---
