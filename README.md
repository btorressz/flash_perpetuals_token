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

### **2️⃣ Staking & Position Management**
#### **stake**
- Allows users to **stake FPT tokens** to gain access to **leveraged trading**.
- Records the **stake timestamp** to enforce **anti-flash-loan protections**.

#### **open_position**
- Opens a **leveraged perpetual position** using the trader’s **staked collateral**.
- Deducts an **execution fee** and **checks margin requirements**.

---


### **3️⃣ Liquidation & Risk Management**
#### **apply_funding_rate**
- Applies **funding payments** between **long and short positions**.

#### **adjust_funding_rate**
- Allows the **admin** to update the **funding rate** dynamically.

#### **liquidate_with_penalty**
- Liquidates **under-margined positions**.
- Applies a **liquidation penalty**, splitting funds between **liquidators and LPs**.

---

### **4️⃣ High-Frequency Trading Enhancements**
#### **batch_execute_trades**
- Enables **batch order execution**, reducing latency for **HFT traders**.

#### **auto_hedge**
- Automatically hedges **large positions** to **protect liquidity providers**.

#### **adjust_leverage_based_on_reputation**
- Dynamically adjusts **leverage limits** based on a trader’s **reputation and trade volume**.

---

## **Security & Protection Mechanisms**
The **Flash Perpetuals Token** system includes **multiple security layers** to prevent **exploits and unfair trading advantages**:

### 🔒 **Access Control**
- **Admin-controlled settings** for **funding rates and liquidation rules**.
- **Role-based permissions** for **market makers, LPs, and liquidation bots**.

### 💸 **Dynamic Liquidation & Partial Liquidation**
- **Prevents full liquidations** by **partially closing** risky positions.
- **Liquidation penalties** ensure **fair rewards for LPs and liquidators**.

### ⚡ **Optimized Execution with Merkle Proofs**
- Uses **Merkle trees** for **order validation**, preventing **order spoofing**.

### 🏦 **Liquidity Pool Management**
- **LP rewards** are **distributed based on trading volume**.
- **Auto-hedging system** protects LP funds **during volatile markets**.

### 🔁 **Auto-Rebalance System**
- Automatically **reallocates liquidity** to **prevent depletion** during **high-volatility events**.

---

