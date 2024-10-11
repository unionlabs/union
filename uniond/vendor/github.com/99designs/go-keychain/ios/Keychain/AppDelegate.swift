//
//  AppDelegate.swift
//  Keychain
//
//  Created by Gabriel on 9/25/15.
//  Copyright © 2015 Gabriel Handford. All rights reserved.
//

import UIKit

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {

  var window: UIWindow?

  func application(_ application: UIApplication, didFinishLaunchingWithOptions launchOptions: [UIApplicationLaunchOptionsKey: Any]?) -> Bool {

    var error: NSError?

    BindAddGenericPassword("KeybaseTest", "gabriel", "A label", "toomanysecrets", nil, &error);
    if (error != nil) {
      print("Failed: \(String(describing: error))")
    } else {
      print("Add OK")
    }

    BindDeleteGenericPassword("KeybaseTest", "gabriel", nil, &error)
    if (error != nil) {
      print("Failed: \(String(describing: error))")
    } else {
      print("Delete OK")
    }

    return true
  }


}

